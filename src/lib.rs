use zed_extension_api::{self as zed, settings::LspSettings, Command, Result};

struct BallerinaExtension;

impl zed::Extension for BallerinaExtension {
    fn new() -> Self {
        BallerinaExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        let mut path = None;
        let mut env = worktree.shell_env();

        // Убедимся, что HOME присутствует в env, так как без него язык не сможет найти ~/.ballerina
        if !env.iter().any(|(k, _)| k == "HOME") {
            if let Ok(home) = std::env::var("HOME") {
                env.push(("HOME".to_string(), home));
            }
        }

        // Попытка прочесть настройки пользователя из settings.json для данного LSP-сервера
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(binary) = settings.binary {
                if let Some(custom_path) = binary.path {
                    path = Some(custom_path);
                }
                if let Some(custom_env) = binary.env {
                    for (key, value) in custom_env {
                        if let Some(pos) = env.iter().position(|(k, _)| k == &key) {
                            env[pos] = (key, value);
                        } else {
                            env.push((key, value));
                        }
                    }
                }
            }
        }

        // Если пользователь не переопределил путь к bal, ищем его в PATH
        let bal_command = if let Some(path) = path {
            path
        } else {
            worktree.which("bal")
                .ok_or_else(|| "The 'bal' command line tool was not found in your PATH. Please install Ballerina Swan Lake or configure the path in settings.".to_string())?
        };

        let node_path = zed::node_binary_path().unwrap_or_else(|_| "node".to_string());

        // Запуск языкового сервера (по умолчанию штатный bal start-language-server)
        let shell_script = format!(
            r#"
WRAPPER_JS="$1"
BAL_BIN=$(readlink -f "{bal_command}" 2>/dev/null || realpath "{bal_command}" 2>/dev/null || echo "{bal_command}")
BAL_INSTALL_DIR=$(dirname "$(dirname "$BAL_BIN")")

# Проверяем, не указал ли пользователь напрямую путь к JAR в настройках
CUSTOM_JAR=""
case "{bal_command}" in
    *.jar) CUSTOM_JAR="{bal_command}" ;;
esac

NODE_BIN="{node_path}"

if [ -n "$CUSTOM_JAR" ] && [ -f "$CUSTOM_JAR" ]; then
    # Ищем JDK в зависимостях Ballerina
    JAVA_PATH=$(ls -1d "$BAL_INSTALL_DIR"/dependencies/jdk-*/bin/java 2>/dev/null | tail -n 1)
    if [ -z "$JAVA_PATH" ] || [ ! -f "$JAVA_PATH" ]; then
        JAVA_PATH="java"
    fi

    # Ищем домашнюю директорию дистрибутива Ballerina
    DIST_HOME=""
    if [ -f "$HOME/.ballerina/ballerina-version" ]; then
        ACTIVE_VER=$(cat "$HOME/.ballerina/ballerina-version" 2>/dev/null | tr -d '\r\n ')
        if [ -n "$ACTIVE_VER" ] && [ -d "$BAL_INSTALL_DIR/distributions/$ACTIVE_VER" ]; then
            DIST_HOME="$BAL_INSTALL_DIR/distributions/$ACTIVE_VER"
        fi
    fi
    if [ -z "$DIST_HOME" ] && [ -f "$BAL_INSTALL_DIR/distributions/ballerina-version" ]; then
        ACTIVE_VER=$(cat "$BAL_INSTALL_DIR/distributions/ballerina-version" 2>/dev/null | tr -d '\r\n ')
        if [ -n "$ACTIVE_VER" ] && [ -d "$BAL_INSTALL_DIR/distributions/$ACTIVE_VER" ]; then
            DIST_HOME="$BAL_INSTALL_DIR/distributions/$ACTIVE_VER"
        fi
    fi
    if [ -z "$DIST_HOME" ]; then
        DIST_HOME=$(ls -1d "$BAL_INSTALL_DIR"/distributions/ballerina-[0-9]* 2>/dev/null | tail -n 1)
    fi
    if [ -z "$DIST_HOME" ]; then
        DIST_HOME="$BAL_INSTALL_DIR"
    fi

    BAL_VER=$(basename "$DIST_HOME" 2>/dev/null | sed -E 's/^ballerina-//' | tr -d '\r\n ')
    LSP_VER=$(basename "$CUSTOM_JAR" 2>/dev/null | sed -E 's/.*ballerina-language-server-([0-9.]+)\.jar/\1/' | tr -d '\r\n ')
    if [ -n "$LSP_VER" ] && [ -n "$BAL_VER" ]; then
        export BAL_LSP_VERSION="$LSP_VER (Ballerina $BAL_VER)"
    elif [ -n "$BAL_VER" ]; then
        export BAL_LSP_VERSION="Ballerina $BAL_VER"
    elif [ -n "$LSP_VER" ]; then
        export BAL_LSP_VERSION="$LSP_VER"
    else
        export BAL_LSP_VERSION="Ballerina"
    fi

    BRE_LIB="$DIST_HOME/bre/lib"
    if [ -d "$BRE_LIB" ]; then
        if [ -n "$NODE_BIN" ] && [ -x "$NODE_BIN" ]; then
            exec "$NODE_BIN" -e "$WRAPPER_JS" "$JAVA_PATH" "-Dballerina.home=$DIST_HOME" "-cp" "$CUSTOM_JAR:$BRE_LIB/*" "org.ballerinalang.langserver.launchers.stdio.Main"
        else
            exec "$JAVA_PATH" "-Dballerina.home=$DIST_HOME" "-cp" "$CUSTOM_JAR:$BRE_LIB/*" "org.ballerinalang.langserver.launchers.stdio.Main"
        fi
    else
        if [ -n "$NODE_BIN" ] && [ -x "$NODE_BIN" ]; then
            exec "$NODE_BIN" -e "$WRAPPER_JS" "$JAVA_PATH" "-Dballerina.home=$DIST_HOME" "-jar" "$CUSTOM_JAR"
        else
            exec "$JAVA_PATH" "-Dballerina.home=$DIST_HOME" "-jar" "$CUSTOM_JAR"
        fi
    fi
else
    BAL_VER=$("{bal_command}" version 2>/dev/null | head -n 1 | awk '{{print $2}}' | tr -d '\r\n ')
    if [ -n "$BAL_VER" ]; then
        export BAL_LSP_VERSION="Ballerina $BAL_VER"
    else
        export BAL_LSP_VERSION="Ballerina"
    fi
    if [ -n "$NODE_BIN" ] && [ -x "$NODE_BIN" ]; then
        exec "$NODE_BIN" -e "$WRAPPER_JS" "{bal_command}" "start-language-server"
    else
        exec "{bal_command}" "start-language-server"
    fi
fi
"#,
            bal_command = bal_command,
            node_path = node_path
        );

        Ok(Command {
            command: "/bin/sh".to_string(),
            args: vec![
                "-c".to_string(),
                shell_script,
                "sh".to_string(),
                LSP_WRAPPER_SCRIPT.to_string(),
            ],
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let mut path = None;
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(binary) = settings.binary {
                if let Some(custom_path) = binary.path {
                    path = Some(custom_path);
                }
            }
        }

        let bal_path = if let Some(path) = path {
            path
        } else {
            worktree.which("bal")
                .ok_or_else(|| "The 'bal' command line tool was not found in your PATH. Please install Ballerina Swan Lake or configure the path in settings.".to_string())?
        };

        let mut user_home = None;
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(init_opts) = settings.initialization_options {
                if let Some(h) = init_opts.get("settings")
                    .and_then(|s| s.get("ballerina"))
                    .and_then(|b| b.get("home"))
                    .and_then(|v| v.as_str()) {
                    user_home = Some(h.to_string());
                } else if let Some(h) = init_opts.get("ballerina")
                    .and_then(|b| b.get("home"))
                    .and_then(|v| v.as_str()) {
                    user_home = Some(h.to_string());
                }
            }
        }

        let ballerina_home = if let Some(home) = user_home {
            home
        } else {
            get_ballerina_home(&bal_path, worktree)
        };

        Ok(Some(zed::serde_json::json!({
            "enableInlayHints": true,
            "settings": {
                "ballerina": {
                    "home": ballerina_home
                }
            }
        })))
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let mut path = None;
        let mut user_home = None;
        if let Ok(settings) = LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            if let Some(binary) = settings.binary {
                if let Some(custom_path) = binary.path {
                    path = Some(custom_path);
                }
            }
            if let Some(init_opts) = settings.initialization_options {
                if let Some(h) = init_opts.get("settings")
                    .and_then(|s| s.get("ballerina"))
                    .and_then(|b| b.get("home"))
                    .and_then(|v| v.as_str()) {
                    user_home = Some(h.to_string());
                } else if let Some(h) = init_opts.get("ballerina")
                    .and_then(|b| b.get("home"))
                    .and_then(|v| v.as_str()) {
                    user_home = Some(h.to_string());
                }
            }
        }

        let bal_path = if let Some(path) = path {
            path
        } else {
            worktree.which("bal")
                .ok_or_else(|| "The 'bal' command line tool was not found in your PATH. Please install Ballerina Swan Lake or configure the path in settings.".to_string())?
        };

        let ballerina_home = if let Some(home) = user_home {
            home
        } else {
            get_ballerina_home(&bal_path, worktree)
        };

        Ok(Some(zed::serde_json::json!({
            "ballerina": {
                "home": ballerina_home
            }
        })))
    }

    fn get_dap_binary(
        &mut self,
        _adapter_name: String,
        config: zed::DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &zed::Worktree,
    ) -> Result<zed::DebugAdapterBinary, String> {
        let bal_path = if let Some(path) = user_provided_debug_adapter_path {
            path
        } else {
            worktree.which("bal")
                .ok_or_else(|| "The 'bal' command line tool was not found in your PATH. Please install Ballerina Swan Lake.".to_string())?
        };

        // Парсим конфигурацию отладки, добавляем ballerina.home, если он не задан пользователем
        let mut launch_config: zed::serde_json::Value = zed::serde_json::from_str(&config.config)
            .unwrap_or_else(|_| zed::serde_json::json!({}));

        if let zed::serde_json::Value::Object(ref mut map) = launch_config {
            if !map.contains_key("ballerina.home") {
                let ballerina_home = get_ballerina_home(&bal_path, worktree);
                map.insert("ballerina.home".to_string(), zed::serde_json::Value::String(ballerina_home));
            }
            
            // Если порт задан как число, переводим его в строку для обхода бага Java/Gson (числа десериализуются как double, превращаясь в "5005.0")
            if let Some(port_val) = map.get("debuggeePort") {
                if let Some(num) = port_val.as_i64() {
                    map.insert("debuggeePort".to_string(), zed::serde_json::Value::String(num.to_string()));
                } else if let Some(f) = port_val.as_f64() {
                    map.insert("debuggeePort".to_string(), zed::serde_json::Value::String((f as i64).to_string()));
                }
            } else {
                map.insert("debuggeePort".to_string(), zed::serde_json::Value::String("5005".to_string()));
            }
        }

        let resolved_config = zed::serde_json::to_string(&launch_config).unwrap_or(config.config);

        let node_path = zed::node_binary_path()?;

        Ok(zed::DebugAdapterBinary {
            command: Some(node_path),
            arguments: vec![
                "-e".to_string(),
                NODE_PROXY_SCRIPT.to_string(),
                bal_path,
            ],
            envs: worktree.shell_env(),
            cwd: None,
            connection: None,
            request_args: zed::StartDebuggingRequestArguments {
                configuration: resolved_config,
                request: zed::StartDebuggingRequestArgumentsRequest::Launch,
            },
        })
    }

    fn dap_request_kind(
        &mut self,
        _adapter_name: String,
        _config: zed::serde_json::Value,
    ) -> Result<zed::StartDebuggingRequestArgumentsRequest, String> {
        Ok(zed::StartDebuggingRequestArgumentsRequest::Launch)
    }
}

const LSP_WRAPPER_SCRIPT: &str = r#"
const { spawn } = require('child_process');

const args = process.argv.slice(1);
if (!args.length) {
    process.exit(1);
}

const serverVersion = process.env.BAL_LSP_VERSION || '1.7.6';
const child = spawn(args[0], args.slice(1), {
    stdio: ['pipe', 'pipe', 'inherit']
});

process.stdin.pipe(child.stdin);
child.stdin.on('error', () => {});
process.stdin.on('error', () => {});
process.stdout.on('error', () => {});

let buffer = Buffer.alloc(0);
let initialized = false;

function onData(chunk) {
    if (initialized) {
        process.stdout.write(chunk);
        return;
    }
    buffer = Buffer.concat([buffer, chunk]);
    while (!initialized) {
        const str = buffer.toString('ascii');
        const headerEnd = str.indexOf('\r\n\r\n');
        if (headerEnd === -1) break;

        const lenMatch = /Content-Length:\s*(\d+)/i.exec(str.substring(0, headerEnd));
        if (!lenMatch) {
            process.stdout.write(buffer);
            buffer = Buffer.alloc(0);
            break;
        }
        const len = parseInt(lenMatch[1], 10);
        const totalMsgLen = headerEnd + 4 + len;
        if (buffer.length < totalMsgLen) break;

        const bodyBuf = buffer.subarray(headerEnd + 4, totalMsgLen);
        const restBuf = buffer.subarray(totalMsgLen);

        try {
            const bodyStr = bodyBuf.toString('utf8');
            const msg = JSON.parse(bodyStr);
            if (msg && msg.result && msg.result.capabilities) {
                if (!msg.result.serverInfo) {
                    msg.result.serverInfo = {
                        name: 'ballerina-language-server',
                        version: serverVersion
                    };
                } else if (!msg.result.serverInfo.version) {
                    msg.result.serverInfo.version = serverVersion;
                }
                const newBody = Buffer.from(JSON.stringify(msg), 'utf8');
                const newHeader = Buffer.from(`Content-Length: ${newBody.length}\r\n\r\n`, 'ascii');
                process.stdout.write(Buffer.concat([newHeader, newBody]));

                initialized = true;
                child.stdout.removeListener('data', onData);
                if (restBuf.length > 0) {
                    process.stdout.write(restBuf);
                }
                child.stdout.pipe(process.stdout);
                break;
            } else {
                process.stdout.write(buffer.subarray(0, totalMsgLen));
                buffer = restBuf;
            }
        } catch (e) {
            process.stdout.write(buffer.subarray(0, totalMsgLen));
            buffer = restBuf;
        }
    }
}

child.stdout.on('data', onData);

child.on('exit', (code) => process.exit(code || 0));
child.on('error', (err) => {
    console.error('LSP wrapper error:', err);
    process.exit(1);
});

process.on('SIGTERM', () => {
    try { child.kill('SIGTERM'); } catch (e) {}
});
process.on('SIGINT', () => {
    try { child.kill('SIGINT'); } catch (e) {}
});
process.on('SIGHUP', () => {
    try { child.kill('SIGHUP'); } catch (e) {}
});
"#;

const NODE_PROXY_SCRIPT: &str = r#"
const fs = require('fs');
const net = require('net');
const spawn = require('child_process').spawn;

function log(...args) {
    const msg = `[${new Date().toISOString()}] ${args.map(x => typeof x === 'object' ? JSON.stringify(x) : String(x)).join(' ')}\n`;
    try {
        fs.appendFileSync('/tmp/ballerina_dap_proxy.log', msg);
    } catch (e) {}
}

log('Proxy starting. argv:', process.argv);

const balPath = process.argv[1];

if (!balPath) {
    log('ERROR: missing arguments');
    process.exit(1);
}

class JsonRpcParser {
    constructor(onMessage) {
        this.buffer = Buffer.alloc(0);
        this.onMessage = onMessage;
    }
    append(chunk) {
        this.buffer = Buffer.concat([this.buffer, chunk]);
        this.process();
    }
    process() {
        while (true) {
            const bufferStr = this.buffer.toString('ascii');
            const contentLengthIndex = bufferStr.indexOf('Content-Length:');
            if (contentLengthIndex === -1) break;
            const headerEndIndex = this.buffer.indexOf('\r\n\r\n', contentLengthIndex);
            if (headerEndIndex === -1) break;
            const lengthStr = bufferStr.substring(contentLengthIndex + 15, headerEndIndex).trim();
            const length = parseInt(lengthStr, 10);
            if (isNaN(length)) {
                this.buffer = Buffer.alloc(0);
                break;
            }
            const messageStartIndex = headerEndIndex + 4;
            if (this.buffer.length < messageStartIndex + length) break;
            const bodyBuffer = this.buffer.subarray(messageStartIndex, messageStartIndex + length);
            this.buffer = this.buffer.subarray(messageStartIndex + length);
            this.onMessage(bodyBuffer.toString('utf8'));
        }
    }
}

function findFreePort(callback) {
    const srv = net.createServer();
    srv.listen(0, '127.0.0.1', () => {
        const port = srv.address().port;
        srv.close(() => callback(port));
    });
}

findFreePort((realPort) => {
    log('Selected realPort:', realPort);
    const child = spawn(balPath, ['start-debugger-adapter', String(realPort)], {
        env: process.env,
        detached: true
    });
    
    child.stdout.on('data', (data) => {
        log('Real DAP stdout:', data.toString().trim());
    });
    child.stderr.on('data', (data) => {
        log('Real DAP stderr:', data.toString().trim());
    });
    child.on('error', (err) => {
        log('Real DAP spawn error:', err);
    });
    child.on('close', (code) => {
        log('Real DAP process closed with code:', code);
        process.exit(code || 0);
    });

    const killChild = () => {
        log('Killing child process group...');
        try {
            process.kill(-child.pid, 'SIGTERM');
        } catch (e) {
            try {
                child.kill('SIGTERM');
            } catch (err) {}
        }
    };

    process.on('exit', () => {
        killChild();
    });
    process.on('SIGTERM', () => {
        log('Received SIGTERM, exiting');
        killChild();
        process.exit(0);
    });
    process.on('SIGINT', () => {
        log('Received SIGINT, exiting');
        killChild();
        process.exit(0);
    });
    process.on('SIGHUP', () => {
        log('Received SIGHUP, exiting');
        killChild();
        process.exit(0);
    });

    let serverSocket = null;
    let pendingClientData = [];
    let connected = false;
    let retries = 0;
    const pendingVariablesEvalReqs = new Map();
    const variablesByName = new Map();
    const childrenCache = new Map();
    let internalSeq = 9000000;
    const internalRequests = new Map();

    function parsePrimitiveValue(rawVal, type) {
        if (rawVal === undefined || rawVal === null) return null;
        if (rawVal === '()') return null;
        if (rawVal === 'true') return true;
        if (rawVal === 'false') return false;
        if (type === 'string' || (rawVal.startsWith('"') && rawVal.endsWith('"') && rawVal.length >= 2)) {
            if (rawVal.startsWith('"') && rawVal.endsWith('"') && rawVal.length >= 2) {
                try {
                    return JSON.parse(rawVal);
                } catch (e) {
                    return rawVal.slice(1, -1);
                }
            }
            return rawVal;
        }
        if (type === 'decimal' || type === 'int' || type === 'float' || type === 'byte') {
            const num = Number(rawVal);
            if (!isNaN(num) && String(num) === rawVal) {
                return num;
            }
            return rawVal;
        }
        return rawVal;
    }

    function fetchVariablesReference(ref) {
        if (childrenCache.has(ref)) {
            return Promise.resolve(childrenCache.get(ref));
        }
        return new Promise((resolve) => {
            const seq = internalSeq++;
            const timer = setTimeout(() => {
                internalRequests.delete(seq);
                resolve([]);
            }, 1500);

            internalRequests.set(seq, (responseMsg) => {
                clearTimeout(timer);
                if (responseMsg && responseMsg.success && responseMsg.body && Array.isArray(responseMsg.body.variables)) {
                    childrenCache.set(ref, responseMsg.body.variables);
                    for (const child of responseMsg.body.variables) {
                        if (child && child.name) {
                            variablesByName.set(child.name, child);
                        }
                    }
                    resolve(responseMsg.body.variables);
                } else {
                    resolve([]);
                }
            });

            const reqObj = {
                type: 'request',
                seq: seq,
                command: 'variables',
                arguments: { variablesReference: ref }
            };
            const bodyStr = JSON.stringify(reqObj);
            const header = `Content-Length: ${Buffer.byteLength(bodyStr, 'utf8')}\r\n\r\n`;
            if (connected && serverSocket && serverSocket.writable) {
                serverSocket.write(header + bodyStr);
            } else {
                clearTimeout(timer);
                internalRequests.delete(seq);
                resolve([]);
            }
        });
    }

    async function resolveVariableValue(varItem, depth = 0, visitedRefs = new Set()) {
        if (!varItem) return null;
        if (!varItem.variablesReference || varItem.variablesReference === 0) {
            return parsePrimitiveValue(varItem.value, varItem.type);
        }
        if (depth > 10 || visitedRefs.has(varItem.variablesReference)) {
            return varItem.value || '[Circular]';
        }
        visitedRefs.add(varItem.variablesReference);

        const children = await fetchVariablesReference(varItem.variablesReference);
        if (!children || children.length === 0) {
            return {};
        }

        const isArray = varItem.type === 'array' || varItem.type === 'tuple' || varItem.type === 'list' ||
            (children.length > 0 && children.every((c, idx) => c.name === `[${idx}]` || c.name === String(idx)));

        if (isArray) {
            const arr = [];
            for (const child of children) {
                arr.push(await resolveVariableValue(child, depth + 1, visitedRefs));
            }
            return arr;
        } else {
            const obj = {};
            for (const child of children) {
                obj[child.name] = await resolveVariableValue(child, depth + 1, visitedRefs);
            }
            return obj;
        }
    }

    const clientParser = new JsonRpcParser((body) => {
        log('Intercepted client request body:', body);
        try {
            const msg = JSON.parse(body);
            if (msg.type === 'request' && msg.command === 'evaluate') {
                if (msg.arguments && (msg.arguments.context === 'variables' || msg.arguments.context === 'clipboard')) {
                    const expr = (msg.arguments.expression || '').trim();
                    pendingVariablesEvalReqs.set(msg.seq, expr);
                    log('Tracked variables/clipboard evaluate request seq:', msg.seq, 'expr:', expr);
                }
            }
        } catch (err) {
            log('Error parsing client request JSON:', err.message);
        }
    });
    
    function connectToRealDap() {
        log('Attempting to connect to real DAP on port', realPort, '(retry', retries, ')');
        serverSocket = net.createConnection({ port: realPort, host: '127.0.0.1' }, () => {
            log('Connected to real DAP');
            connected = true;
            for (const chunk of pendingClientData) {
                serverSocket.write(chunk);
            }
            pendingClientData = [];
        });
        
        serverSocket.on('error', (err) => {
            log('Real DAP socket error:', err.message);
            if (retries < 50) {
                retries++;
                setTimeout(connectToRealDap, 100);
            } else {
                log('Connection to real DAP failed after 50 retries');
                process.exit(1);
            }
        });
        
        const parser = new JsonRpcParser((body) => {
            log('Intercepted response body:', body);
            let modifiedBody = body.replace(/"file:\/\/([^"]*)"/g, (match, rawPath) => {
                let decoded = rawPath;
                try {
                    decoded = decodeURIComponent(rawPath);
                } catch (e) {}
                if (decoded.startsWith('/') && decoded.match(/^\/[a-zA-Z]:/)) {
                    decoded = decoded.substring(1);
                }
                return JSON.stringify(decoded);
            });
            
            try {
                const msg = JSON.parse(modifiedBody);

                // Handle internal proxy responses (e.g. recursive variable fetches)
                if (msg.type === 'response' && internalRequests.has(msg.request_seq)) {
                    const handler = internalRequests.get(msg.request_seq);
                    internalRequests.delete(msg.request_seq);
                    handler(msg);
                    return; // Do NOT forward internal responses to Zed!
                }

                // Cache variables from normal variables responses
                if (msg.type === 'response' && msg.command === 'variables' && msg.body && Array.isArray(msg.body.variables)) {
                    for (const v of msg.body.variables) {
                        if (v && v.name) {
                            variablesByName.set(v.name, v);
                        }
                    }
                }

                // Clear caches when execution continues
                if (msg.type === 'event' && (msg.event === 'continued' || msg.event === 'stopped')) {
                    variablesByName.clear();
                    childrenCache.clear();
                }

                // Intercept evaluate response from Zed for variables/clipboard context
                if (msg.type === 'response' && msg.command === 'evaluate' && pendingVariablesEvalReqs.has(msg.request_seq)) {
                    const expr = pendingVariablesEvalReqs.get(msg.request_seq);
                    pendingVariablesEvalReqs.delete(msg.request_seq);
                    const lookupName = expr.includes('.') ? expr.split('.').pop().trim() : expr;
                    const varItem = variablesByName.get(expr) || variablesByName.get(lookupName);

                    if (varItem) {
                        log('Resolving variable for evaluate response:', expr, varItem.type, varItem.variablesReference);
                        resolveVariableValue(varItem).then((resolved) => {
                            let resultStr;
                            if (typeof resolved === 'string') {
                                resultStr = resolved;
                            } else if (resolved === null) {
                                resultStr = varItem.type === 'nil' ? '()' : 'null';
                            } else if (typeof resolved === 'object') {
                                resultStr = JSON.stringify(resolved, null, 2);
                            } else {
                                resultStr = String(resolved);
                            }
                            log('Successfully resolved variable value for', expr, ':', resultStr.slice(0, 100));
                            msg.success = true;
                            msg.body = { result: resultStr, variablesReference: 0 };
                            const resStr = JSON.stringify(msg);
                            const response = `Content-Length: ${Buffer.byteLength(resStr, 'utf8')}\r\n\r\n${resStr}`;
                            process.stdout.write(response);
                        }).catch((err) => {
                            log('Error resolving variable value:', err);
                            msg.success = false;
                            msg.message = 'Evaluation not supported for variables context';
                            delete msg.body;
                            const resStr = JSON.stringify(msg);
                            const response = `Content-Length: ${Buffer.byteLength(resStr, 'utf8')}\r\n\r\n${resStr}`;
                            process.stdout.write(response);
                        });
                        return; // Async response sent inside Promise!
                    } else {
                        log('Variable not found in cache for expression:', expr);
                        msg.success = false;
                        msg.message = 'Evaluation not supported for variables context';
                        delete msg.body;
                        modifiedBody = JSON.stringify(msg);
                    }
                } else if (msg.type === 'response' && (msg.command === 'terminate' || msg.command === 'disconnect') && msg.success) {
                    log(`Detected successful ${msg.command} response. Scheduling exit...`);
                    setTimeout(() => {
                        log('Exiting after terminate/disconnect response');
                        killChild();
                        process.exit(0);
                    }, 200);
                }
            } catch (err) {
                log('Error parsing response JSON:', err.message);
            }

            log('Modified response body:', modifiedBody);
            const response = `Content-Length: ${Buffer.byteLength(modifiedBody, 'utf8')}\r\n\r\n${modifiedBody}`;
            process.stdout.write(response);
        });
        
        serverSocket.on('data', (chunk) => parser.append(chunk));
        serverSocket.on('end', () => {
            log('Real DAP socket ended');
            process.exit(0);
        });
    }
    
    connectToRealDap();
    
    process.stdin.on('data', (chunk) => {
        log('Received data from Zed:', chunk.toString('utf8'));
        clientParser.append(chunk);
        if (connected && serverSocket && serverSocket.writable) {
            serverSocket.write(chunk);
        } else {
            pendingClientData.push(chunk);
        }
    });
    
    process.stdin.on('error', (err) => {
        log('Zed stdin error:', err.message);
    });
    process.stdin.on('end', () => {
        log('Zed stdin ended, exiting...');
        killChild();
        process.exit(0);
    });
});
"#;


fn get_ballerina_home(bal_path: &str, worktree: &zed::Worktree) -> String {
    let mut env = worktree.shell_env();
    if !env.iter().any(|(k, _)| k == "HOME") {
        if let Ok(home) = std::env::var("HOME") {
            env.push(("HOME".to_string(), home));
        }
    }

    let mut cmd = Command::new("/bin/sh".to_string())
        .args(vec![
            "-c".to_string(),
            format!(
                r#"
                if [ -f "$HOME/.ballerina/ballerina-version" ]; then
                    ACTIVE_VER=$(cat "$HOME/.ballerina/ballerina-version" 2>/dev/null | tr -d '\r\n ')
                    if [ -n "$ACTIVE_VER" ] && [ -d "/Library/Ballerina/distributions/$ACTIVE_VER" ]; then
                        echo "/Library/Ballerina/distributions/$ACTIVE_VER"
                        exit 0
                    fi
                fi
                BAL_BIN=$(readlink -f "{bal_path}" 2>/dev/null || realpath "{bal_path}" 2>/dev/null || echo "{bal_path}")
                BAL_INSTALL_DIR=$(dirname "$(dirname "$BAL_BIN")")
                if [ -f "$BAL_INSTALL_DIR/distributions/ballerina-version" ]; then
                    ACTIVE_VER=$(cat "$BAL_INSTALL_DIR/distributions/ballerina-version" 2>/dev/null | tr -d '\r\n ')
                    if [ -n "$ACTIVE_VER" ] && [ -d "$BAL_INSTALL_DIR/distributions/$ACTIVE_VER" ]; then
                        echo "$BAL_INSTALL_DIR/distributions/$ACTIVE_VER"
                        exit 0
                    fi
                fi
                LATEST=$(ls -1d "$BAL_INSTALL_DIR"/distributions/ballerina-[0-9]* 2>/dev/null | tail -n 1)
                if [ -n "$LATEST" ] && [ -d "$LATEST" ]; then
                    echo "$LATEST"
                    exit 0
                fi
                if [ -d "$BAL_INSTALL_DIR" ] && [ -d "$BAL_INSTALL_DIR/bre" ]; then
                    echo "$BAL_INSTALL_DIR"
                    exit 0
                fi
                LATEST_SYS=$(ls -1d /Library/Ballerina/distributions/ballerina-[0-9]* 2>/dev/null | tail -n 1)
                if [ -n "$LATEST_SYS" ] && [ -d "$LATEST_SYS" ]; then
                    echo "$LATEST_SYS"
                    exit 0
                fi
                echo "/Library/Ballerina/distributions/ballerina-2201.13.6"
                "#
            ),
        ])
        .envs(env);

    if let Ok(output) = cmd.output() {
        if output.status == Some(0) {
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !out.is_empty() {
                return out;
            }
        }
    }

    if bal_path.contains("/opt/homebrew") {
        "/opt/homebrew/opt/ballerina/libexec".to_string()
    } else {
        "/Library/Ballerina/distributions/ballerina-2201.13.6".to_string()
    }
}

zed::register_extension!(BallerinaExtension);


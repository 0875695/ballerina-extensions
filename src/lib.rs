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

        // Запуск через /bin/sh с использованием shell-скрипта для поиска бандлированного JAR языкового сервера
        let shell_script = format!(
            r#"
BAL_BIN=$(readlink -f "{bal_command}" 2>/dev/null || realpath "{bal_command}" 2>/dev/null || echo "{bal_command}")
BAL_INSTALL_DIR=$(dirname "$(dirname "$BAL_BIN")")

# Ищем бандлированную версию языкового сервера
JAR_PATH=$(ls -1d "$HOME"/.antigravity-ide/extensions/wso2.ballerina-*/ls/ballerina-language-server-*.jar "$HOME"/.vscode/extensions/wso2.ballerina-*/ls/ballerina-language-server-*.jar "$HOME"/.cursor/extensions/wso2.ballerina-*/ls/ballerina-language-server-*.jar 2>/dev/null | tail -n 1)

if [ -n "$JAR_PATH" ] && [ -f "$JAR_PATH" ]; then
    # Ищем JDK в зависимостях Ballerina (выбираем самую свежую версию с помощью tail -n 1)
    JAVA_PATH=$(ls -1d "$BAL_INSTALL_DIR"/dependencies/jdk-*/bin/java 2>/dev/null | tail -n 1)
    if [ -z "$JAVA_PATH" ] || [ ! -f "$JAVA_PATH" ]; then
        JAVA_PATH="java"
    fi
    
    # Ищем домашнюю директорию дистрибутива Ballerina (фильтруем по шаблону [0-9]*, чтобы исключить файл ballerina-version)
    DIST_HOME=$(ls -1d "$BAL_INSTALL_DIR"/distributions/ballerina-[0-9]* 2>/dev/null | tail -n 1)
    if [ -z "$DIST_HOME" ]; then
        DIST_HOME="$BAL_INSTALL_DIR"
    fi
    
    exec "$JAVA_PATH" "-Dballerina.home=$DIST_HOME" "-jar" "$JAR_PATH"
else
    exec "{bal_command}" "start-language-server"
fi
"#,
            bal_command = bal_command
        );

        Ok(Command {
            command: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), shell_script],
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

        let ballerina_home = get_ballerina_home(&bal_path);

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

        let ballerina_home = get_ballerina_home(&bal_path);

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
                let ballerina_home = get_ballerina_home(&bal_path);
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
            const modifiedBody = body.replace(/"file:\/\/([^"]*)"/g, (match, path) => {
                if (path.startsWith('/') && path.match(/^\/[a-zA-Z]:/)) {
                    return `"${path.substring(1)}"`;
                }
                return `"${path}"`;
            });
            log('Modified response body:', modifiedBody);
            const response = `Content-Length: ${Buffer.byteLength(modifiedBody, 'utf8')}\r\n\r\n${modifiedBody}`;
            process.stdout.write(response);
            
            try {
                const msg = JSON.parse(body);
                if (msg.type === 'response' && (msg.command === 'terminate' || msg.command === 'disconnect') && msg.success) {
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


fn get_ballerina_home(bal_path: &str) -> String {
    let mut cmd = Command::new(bal_path.to_string());
    cmd.args = vec!["version".to_string()];
    
    let mut version = "2201.13.4".to_string();
    if let Ok(output) = cmd.output() {
        if output.status == Some(0) {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    version = parts[1].to_string();
                }
            }
        }
    }

    if bal_path.contains("/Library/Ballerina") {
        format!("/Library/Ballerina/distributions/ballerina-{}", version)
    } else if let Some(parent) = std::path::Path::new(bal_path).parent().and_then(|p| p.parent()) {
        let parent_str = parent.to_string_lossy().to_string();
        if parent_str.contains("/opt/homebrew") {
            "/opt/homebrew/opt/ballerina/libexec".to_string()
        } else {
            format!("{}/distributions/ballerina-{}", parent_str, version)
        }
    } else {
        format!("/Library/Ballerina/distributions/ballerina-{}", version)
    }
}

zed::register_extension!(BallerinaExtension);


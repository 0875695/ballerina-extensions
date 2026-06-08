const fs = require('fs');

const data = fs.readFileSync('/Users/oivanov/.vscode/extensions/wso2.ballerina-5.12.0/grammar/ballerina-grammar/syntaxes/ballerina.tmLanguage', 'utf8');

const matches = [];
const keywords = new Set();

// Regular expression to find <key>match</key>\s*<string>(.*?)</string>
const matchRegex = /<key>match<\/key>\s*<string>(.*?)<\/string>/g;
let match;
while ((match = matchRegex.exec(data)) !== null) {
    const pattern = match[1];
    matches.push(pattern);
    
    // Look for \b(word1|word2)\b patterns
    const wordRegex = /\\b\(([^)]+)\)\\b/g;
    let wordMatch;
    while ((wordMatch = wordRegex.exec(pattern)) !== null) {
        wordMatch[1].split('|').forEach(w => {
            // Filter out non-alphabetic/regex noise
            if (/^[a-zA-Z0-9_]+$/.test(w)) {
                keywords.add(w);
            }
        });
    }
}

console.log("Found keywords:");
console.log(Array.from(keywords).sort().join(', '));

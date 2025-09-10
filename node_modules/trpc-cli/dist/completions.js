"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.addCompletions = addCompletions;
/** uses omelette to add completions to a commander program */
function addCompletions(program, completion) {
    const commandSymbol = Symbol('command');
    const cTree = {};
    function addCommandCompletions(command, cTreeNode) {
        command.commands.forEach(c => {
            const node = (cTreeNode[c.name()] ||= {});
            Object.defineProperty(node, commandSymbol, { value: c, enumerable: false });
            addCommandCompletions(c, node);
        });
    }
    addCommandCompletions(program, cTree);
    completion.on('complete', (fragment, params) => {
        const segments = params.line.split(/ +/).slice(1, params.fragment);
        const last = segments.at(-1);
        let node = cTree;
        const existingFlags = new Set();
        for (const segment of segments) {
            if (segment.startsWith('-')) {
                existingFlags.add(segment);
                continue;
            }
            if (existingFlags.size > 0)
                continue;
            node = node[segment];
            if (!node)
                return;
        }
        const correspondingCommand = node[commandSymbol];
        if (correspondingCommand?.options?.length) {
            const suggestions = [];
            for (const o of correspondingCommand.options) {
                if (last === o.long || last === o.short) {
                    if (o.argChoices)
                        suggestions.push(...o.argChoices);
                    if (!o.isBoolean())
                        break;
                }
                if (existingFlags.has(o.long))
                    continue;
                if (existingFlags.has(o.short))
                    continue;
                suggestions.push(o.long);
            }
            return void params.reply(suggestions);
        }
    });
    completion.tree(cTree).init();
}

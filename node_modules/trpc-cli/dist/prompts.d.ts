import { Argument, Command, Option } from 'commander';
import { CommanderProgramLike, Promptable } from './types';
type Shadowed<T> = {
    original: T;
    shadow: T;
};
type WithValue<T> = Shadowed<T> & {
    value: string | undefined;
    specified: boolean;
};
type Analysis = {
    command: Shadowed<Command>;
    arguments: WithValue<Argument>[];
    options: WithValue<Option>[];
};
export declare const createShadowCommand: (command: Command, onAnalyze: (params: Analysis) => void | Promise<void>) => Command;
export declare const promptify: (program: CommanderProgramLike, prompts: Promptable) => CommanderProgramLike;
export {};

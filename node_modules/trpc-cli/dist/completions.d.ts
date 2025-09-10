import { OmeletteInstanceLike } from '.';
import { Command } from 'commander';
/** uses omelette to add completions to a commander program */
export declare function addCompletions(program: Command, completion: OmeletteInstanceLike): void;

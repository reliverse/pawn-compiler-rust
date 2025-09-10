import { Logger } from './types';
export declare const lineByLineLogger: (logger: Logger) => Logger;
/**
 * A logger which uses `console.log` and `console.error` to log in the following way:
 * - Primitives are logged directly
 * - Arrays are logged item-by-item
 * - Objects are logged as JSON
 *
 * This is useful for logging structured data in a human-readable way, and for piping logs to other tools.
 */
export declare const lineByLineConsoleLogger: Logger;

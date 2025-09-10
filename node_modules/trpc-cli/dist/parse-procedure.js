"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseProcedureInputs = parseProcedureInputs;
const util_1 = require("util");
const zod4 = __importStar(require("zod/v4/core"));
const zod_to_json_schema_1 = __importDefault(require("zod-to-json-schema"));
const errors_1 = require("./errors");
const json_schema_1 = require("./json-schema");
// We're going to use eval to require some optional dependencies. It's hard-coded, so safe, but some bundlers like tsdown will emit warnings unless we disguise it.
const disguisedEval = eval;
/**
 * Attempts to convert a trpc procedure input to JSON schema.
 * Uses @see jsonSchemaConverters to convert the input to JSON schema.
 */
function toJsonSchema(input, dependencies) {
    try {
        const jsonSchemaConverters = getJsonSchemaConverters(dependencies);
        const vendor = getVendor(input);
        if (vendor && vendor in jsonSchemaConverters) {
            const converter = jsonSchemaConverters[vendor];
            const converted = converter(input);
            return { success: true, value: converted };
        }
        return { success: false, error: `Schema not convertible to JSON schema` };
    }
    catch (e) {
        const message = e instanceof Error ? e.message : String(e);
        return { success: false, error: `Failed to convert input to JSON Schema: ${message}` };
    }
}
function looksLikeJsonSchema(value) {
    return (typeof value === 'object' &&
        value !== null &&
        (('type' in value && (typeof value.type === 'string' || Array.isArray(value.type))) ||
            'const' in value ||
            'oneOf' in value ||
            'anyOf' in value));
}
function parseProcedureInputs(inputs, dependencies) {
    const inner = parseProcedureInputsInner(inputs, dependencies);
    if (inner.success && inner.value.positionalParameters.some((param, i, { length }) => param.array && i < length - 1)) {
        return { success: false, error: `Array positional parameters must be at the end of the input.` };
    }
    if (inner.success) {
        const optionsProps = schemaDefPropValue(inner.value.optionsJsonSchema, 'properties');
        if (optionsProps) {
            const optionishPositionals = Object.entries(optionsProps).flatMap(([key, schema]) => {
                if (typeof schema === 'object' && 'positional' in schema && schema.positional === true) {
                    return [{ key, schema }];
                }
                return [];
            });
            if (optionishPositionals.length > 0) {
                return {
                    success: true,
                    value: {
                        positionalParameters: [
                            ...inner.value.positionalParameters,
                            ...optionishPositionals.map(({ key, schema }) => ({
                                name: key,
                                array: looksLikeArray(schema),
                                description: schema.description ?? '',
                                required: !isOptional(schema),
                                type: (0, json_schema_1.getSchemaTypes)(schema).join(' | '),
                            })),
                        ],
                        optionsJsonSchema: {
                            ...inner.value.optionsJsonSchema,
                            properties: Object.fromEntries(Object.entries(optionsProps).filter(([key]) => !optionishPositionals.some(x => x.key === key))),
                        },
                        getPojoInput: params => {
                            const positionalValues = [...params.positionalValues];
                            const options = { ...params.options };
                            for (const { key, schema } of optionishPositionals) {
                                options[key] = convertPositional(schema, positionalValues.shift());
                            }
                            return inner.value.getPojoInput({ positionalValues, options });
                        },
                    },
                };
            }
        }
    }
    return inner;
}
function parseProcedureInputsInner(inputs, dependencies) {
    if (inputs.length === 0) {
        return {
            success: true,
            value: {
                positionalParameters: [],
                optionsJsonSchema: {},
                getPojoInput: () => ({}),
            },
        };
    }
    const allJsonSchemaable = inputs.every(input => looksJsonSchemaable(input));
    if (!allJsonSchemaable) {
        return {
            success: false,
            error: `Invalid input type ${inputs.map(s => s?.constructor.name).join(', ')}, only inputs that can be converted to JSON Schema are supported`,
        };
    }
    if (inputs.length > 1) {
        return parseMultiInputs(inputs, dependencies);
    }
    const mergedSchemaResult = toJsonSchema(inputs[0], dependencies);
    if (!mergedSchemaResult.success) {
        return {
            success: false,
            error: mergedSchemaResult.error,
        };
    }
    const mergedSchema = mergedSchemaResult.value;
    return handleMergedSchema(mergedSchema);
}
function handleMergedSchema(mergedSchema) {
    if (mergedSchema.additionalProperties) {
        return { success: false, error: `Inputs with additional properties are not currently supported` };
    }
    if (acceptedPrimitiveTypes(mergedSchema).length > 0) {
        return parsePrimitiveInput(mergedSchema);
    }
    if (isTuple(mergedSchema)) {
        return parseTupleInput(mergedSchema);
    }
    if (mergedSchema.type === 'array') {
        return parseArrayInput(mergedSchema);
    }
    if (mergedSchema.anyOf) {
        const allObjects = mergedSchema.anyOf.every(sub => acceptsObject(toRoughJsonSchema7(sub)));
        if (allObjects) {
            return {
                success: true,
                value: {
                    positionalParameters: [],
                    optionsJsonSchema: mergedSchema,
                    getPojoInput: argv => argv.options,
                },
            };
        }
        if (mergedSchema.anyOf.length === 2 && JSON.stringify(mergedSchema.anyOf[0]) === '{"not":{}}') {
            return handleMergedSchema(mergedSchema.anyOf[1]);
        }
    }
    if (mergedSchema.type !== 'object') {
        return {
            success: false,
            error: `Invalid input type ${(0, util_1.inspect)(mergedSchema, { depth: 2, breakLength: Infinity })}, expected object or tuple.`,
        };
    }
    return {
        success: true,
        value: {
            positionalParameters: [],
            optionsJsonSchema: mergedSchema,
            getPojoInput: argv => argv.options,
        },
    };
}
// zod-to-json-schema turns `z.string().optional()` into `{"anyOf":[{"not":{}},{"type":"string"}]}`
function isOptional(schema) {
    if (schema && typeof schema === 'object' && 'optional' in schema)
        return schema.optional === true;
    if (schemaDefPropValue(schema, 'not') && JSON.stringify(schema) === '{"not":{}}')
        return true;
    const anyOf = schemaDefPropValue(schema, 'anyOf');
    if (anyOf?.some(sub => isOptional(sub)))
        return true;
    if (schemaDefPropValue(schema, 'default') !== undefined)
        return true;
    return false;
}
function parsePrimitiveInput(schema) {
    const typeName = acceptedPrimitiveTypes(schema).join(' | ');
    const name = (schema.title || schema.description || /\W/.test(typeName) ? 'value' : typeName).replaceAll(/\s+/g, '_');
    return {
        success: true,
        value: {
            positionalParameters: [
                {
                    name,
                    array: false,
                    description: schema.description || '',
                    required: !isOptional(schema),
                    type: typeName,
                },
            ],
            optionsJsonSchema: {},
            getPojoInput: argv => convertPositional(schema, argv.positionalValues[0]),
        },
    };
}
const schemaDefPropValue = (schema, prop) => {
    if (schema && typeof schema === 'object' && prop in schema)
        return schema[prop];
    return undefined;
};
const primitiveCandidateTypes = ['string', 'number', 'boolean', 'integer'];
function acceptedPrimitiveTypes(schema) {
    let constVals = [toRoughJsonSchema7(schema).const, toRoughJsonSchema7(schema).enum]
        .flat()
        .filter(Boolean)
        .map(s => typeof s);
    if (constVals.length === 0)
        constVals = undefined;
    const typeList = constVals ||
        schemaDefPropValue(schema, 'type') ||
        schemaDefPropValue(schema, 'oneOf')?.flatMap(s => acceptedPrimitiveTypes(s)) ||
        schemaDefPropValue(schema, 'anyOf')?.flatMap(s => acceptedPrimitiveTypes(s));
    const acceptedJsonSchemaTypes = new Set([typeList].flat().filter(Boolean));
    return primitiveCandidateTypes.filter(c => acceptedJsonSchemaTypes.has(c));
}
/**
 * From a list of schemas, if they are all record-style schemas, return a single schema with all properties (an intersection).
 * Returns `null` if the schemas are not all record-style schemas.
 */
function maybeMergeObjectSchemas(schemas) {
    const required = [];
    const properties = {};
    for (const schema of schemas) {
        if (!schema)
            return null;
        const { required: schemaRequired, properties: schemaProperties, type, $schema, ...rest } = schema;
        if (type && type !== 'object')
            return null;
        if (Object.keys(rest).length)
            return null;
        if (schemaRequired)
            required.push(...schemaRequired);
        if (schemaProperties)
            Object.assign(properties, schemaProperties);
    }
    return { type: 'object', required, properties };
}
function parseMultiInputs(inputs, dependencies) {
    const parsedIndividually = inputs.map(input => parseProcedureInputsInner([input], dependencies));
    const failures = parsedIndividually.flatMap(p => (p.success ? [] : [p.error]));
    if (failures.length > 0) {
        return { success: false, error: failures.join('\n') };
    }
    const allObjects = parsedIndividually.every(p => p.success && p.value.positionalParameters.length === 0);
    if (!allObjects) {
        return {
            success: false,
            error: `Can't use positional parameters with multi-input type.`,
        };
    }
    const merged = maybeMergeObjectSchemas(parsedIndividually.map(p => (p.success ? p.value.optionsJsonSchema : {})));
    if (merged) {
        return {
            success: true,
            value: {
                positionalParameters: [],
                optionsJsonSchema: merged,
                getPojoInput: argv => argv.options,
            },
        };
    }
    return {
        success: true,
        value: {
            positionalParameters: [],
            optionsJsonSchema: {
                allOf: parsedIndividually.map(p => {
                    const successful = p;
                    const optionsSchema = successful.value.optionsJsonSchema;
                    if ('additionalProperties' in optionsSchema && optionsSchema.additionalProperties === false) {
                        const { additionalProperties, ...rest } = optionsSchema;
                        return rest;
                    }
                    return optionsSchema;
                }),
            },
            getPojoInput: argv => argv.options,
        },
    };
}
function isNullable(schema) {
    if (Array.isArray(schema.type) && schema.type.includes('null'))
        return true;
    if (schema.type === 'null')
        return true;
    if ((schema.anyOf || schema.oneOf)?.some(sub => isNullable(toRoughJsonSchema7(sub))))
        return true;
    if (schema.const === null)
        return true;
    return false;
}
const tupleItemsSchemas = (schema) => {
    if (!schema || typeof schema !== 'object')
        return undefined;
    if (Array.isArray(schema.items))
        return schema.items;
    if ('prefixItems' in schema && Array.isArray(schema.prefixItems))
        return schema.prefixItems;
    return undefined;
};
function isTuple(schema) {
    return Array.isArray(tupleItemsSchemas(schema));
}
function parseArrayInput(array) {
    if (looksLikeJsonSchema(array.items) && isNullable(array.items)) {
        return {
            success: false,
            error: `Invalid input type Array<${(0, json_schema_1.getSchemaTypes)(array.items).join(' | ')}>. Nullable arrays are not supported.`,
        };
    }
    return {
        success: true,
        value: {
            positionalParameters: [
                {
                    name: parameterName(array, 1),
                    array: true,
                    description: array.description || '',
                    required: !isOptional(array),
                    type: 'string',
                },
            ],
            optionsJsonSchema: {},
            getPojoInput: argv => argv.positionalValues.at(-1).map(s => convertPositional(array.items, s)),
        },
    };
}
function parseTupleInput(tuple) {
    const items = tupleItemsSchemas(tuple);
    if (!Array.isArray(items))
        throw new Error('.items is not an array, is this really a tuple?');
    const flagsSchemaIndex = items.findIndex(item => {
        if (acceptedPrimitiveTypes(item).length > 0) {
            return false; // it's a string, number or boolean
        }
        if (looksLikeArray(item) && acceptedPrimitiveTypes(item.items).length > 0) {
            return false; // it's an array of strings, numbers or booleans
        }
        return true; // it's not a string, number, boolean or array of strings, numbers or booleans. So it's probably a flags object
    });
    const types = `[${items.map(s => schemaDefPropValue(s, 'type')).join(', ')}]`;
    if (flagsSchemaIndex > -1 && flagsSchemaIndex !== items.length - 1) {
        return {
            success: false,
            error: `Invalid input type ${types}. Positional parameters must be strings, numbers or booleans.`,
        };
    }
    const flagsSchema = flagsSchemaIndex === -1 ? null : items[flagsSchemaIndex];
    if (flagsSchema && !acceptsObject(flagsSchema)) {
        return {
            success: false,
            error: `Invalid input type ${types}. The last type must accept object inputs.`,
        };
    }
    const positionalSchemas = flagsSchemaIndex === -1 ? items : items.slice(0, flagsSchemaIndex);
    return {
        success: true,
        value: {
            positionalParameters: positionalSchemas.map((schema, i) => ({
                name: parameterName(schema, i + 1),
                array: looksLikeArray(schema),
                description: schemaDefPropValue(schema, 'description') || '',
                required: !isOptional(schema),
                type: (0, json_schema_1.getSchemaTypes)(toRoughJsonSchema7(schema)).join(' | '),
            })),
            optionsJsonSchema: flagsSchema && typeof flagsSchema === 'object' ? flagsSchema : {},
            getPojoInput: commandArgs => {
                const inputs = commandArgs.positionalValues.map((v, i) => {
                    const correspondingSchema = positionalSchemas[i];
                    if (looksLikeArray(correspondingSchema)) {
                        if (!Array.isArray(v)) {
                            throw new errors_1.CliValidationError(`Expected array at position ${i}, got ${typeof v}`);
                        }
                        return v.map(s => {
                            if (!correspondingSchema.items || Array.isArray(correspondingSchema.items))
                                return s;
                            return convertPositional(correspondingSchema.items, s);
                        });
                    }
                    if (typeof v !== 'string' && v !== undefined) {
                        throw new errors_1.CliValidationError(`Expected string at position ${i}, got ${typeof v}`);
                    }
                    return convertPositional(correspondingSchema, v);
                });
                if (flagsSchema) {
                    inputs.push(commandArgs.options);
                }
                return inputs;
            },
        },
    };
}
/**
 * Converts a positional string to parameter into a number if the target schema accepts numbers, and the input can be parsed as a number.
 * If the target schema accepts numbers but it's *not* a valid number, just return a string.
 * trpc will use zod to handle the validation before invoking the procedure.
 */
const convertPositional = (schema, value) => {
    let preprocessed = undefined;
    const acceptedTypes = new Set(acceptedPrimitiveTypes(schema));
    if (acceptedTypes.has('string')) {
        preprocessed = value;
    }
    if (acceptedTypes.has('boolean')) {
        if (value === 'true')
            preprocessed = true;
        else if (value === 'false')
            preprocessed = false;
    }
    if (acceptedTypes.has('number')) {
        const number = Number(value);
        if (!Number.isNaN(number)) {
            preprocessed = number;
        }
    }
    if (acceptedTypes.has('integer')) {
        const num = Number(value);
        if (Number.isInteger(num)) {
            preprocessed = num;
        }
        else if (!Number.isNaN(num) && acceptedTypes === undefined) {
            // we're expecting an integer and the value isn't one, but we haven't come up with anything else, so use it anyway to get helpful "expected integer, got float" error rather than "expected number, got string"
            preprocessed = value;
        }
    }
    if (preprocessed === undefined) {
        return value; // we didn't convert to a number or boolean, so just return the string
    }
    return preprocessed;
};
const looksLikeArray = (schema) => {
    return schemaDefPropValue(schema, 'type') === 'array';
};
const toRoughJsonSchema7 = (schema) => {
    if (!schema || typeof schema !== 'object') {
        return {};
    }
    return schema;
};
const maybeParameterName = (s) => {
    const value = schemaDefPropValue(s, 'title') || schemaDefPropValue(s, 'description');
    // only look at array item title if we don't have one for the outer array itself
    // e.g. for {title: 'file collection', items: {title: 'file'}} we prefer 'file collection' as the parameter name
    if (!value && looksLikeArray(s)) {
        const items = toRoughJsonSchema7(s).items;
        return items && !Array.isArray(items) ? maybeParameterName(items) : undefined;
    }
    return value;
};
const parameterName = (s, position) => {
    let name = maybeParameterName(s) || `parameter_${position}`;
    if (looksLikeArray(s))
        return `[${name}...]`;
    // commander requiremenets: no special characters in positional parameters; `<name>` for required and `[name]` for optional parameters
    name = name.replaceAll(/\W+/g, ' ').trim();
    return isOptional(s) ? `[${name}]` : `<${name}>`;
};
const acceptsObject = (schema) => {
    return (schema.type === 'object' || schema.anyOf?.some(sub => acceptsObject(toRoughJsonSchema7(sub)))) ?? false;
};
// #region vendor specific stuff
/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-require-imports */
/** `Record<standard-schema vendor id, function that converts the input to JSON schema>` */
const getJsonSchemaConverters = (dependencies) => {
    return {
        zod: (input) => {
            // @ts-expect-error don't worry lots of ?.
            if (input._zod?.version?.major == 4) {
                return zod4.toJSONSchema(input, {
                    // todo[zod@>=4.0.0] remove the line if https://github.com/colinhacks/zod/issues/4167 is resolved, or this comment if it's closed
                    io: 'input',
                    // todo[zod@>=4.0.0] remove the override if https://github.com/colinhacks/zod/issues/4164 is resolved, or this comment if it's closed
                    unrepresentable: 'any',
                    // todo[zod@>=4.0.0] remove the override if https://github.com/colinhacks/zod/issues/4164 is resolved, or this comment if it's closed
                    override: ctx => {
                        if (ctx.zodSchema?.constructor?.name === 'ZodOptional') {
                            ctx.jsonSchema.optional = true;
                        }
                        // this is needed because trpc-cli (currently) has its own zod dependency, which is v3, and uses zod/v4 as a submodule. But the v3 zod/v4 module drops descriptions from the produced json schema.
                        // normally zod does this itself, but not when using v3's toJSONSchema function with a v4 schema.
                        const meta = ctx.zodSchema.meta?.();
                        if (meta)
                            Object.assign(ctx.jsonSchema, meta);
                    },
                });
            }
            return (0, zod_to_json_schema_1.default)(input);
        },
        arktype: (input) => {
            const type = prepareArktypeType(input);
            return type.toJsonSchema({
                fallback: ctx => {
                    if (ctx.code === 'unit' && ctx.unit === undefined)
                        return { ...ctx.base, optional: true };
                    return ctx.base;
                },
            });
        },
        valibot: (input) => {
            let valibotToJsonSchemaLib = dependencies['@valibot/to-json-schema'];
            if (!valibotToJsonSchemaLib) {
                try {
                    valibotToJsonSchemaLib = disguisedEval(`require('@valibot/to-json-schema')`);
                }
                catch (e) {
                    throw new Error(`@valibot/to-json-schema could not be found - try installing it and re-running`, { cause: e });
                }
            }
            const valibotToJsonSchema = valibotToJsonSchemaLib?.toJsonSchema;
            if (!valibotToJsonSchema) {
                throw new Error(`no 'toJsonSchema' function found in @valibot/to-json-schema - check you are using a supported version`);
            }
            let v;
            try {
                v = disguisedEval(`require('valibot')`);
            }
            catch {
                // couldn't load valibot, maybe it's aliased to something else? anyway bad luck, you won't know about optional positional parameters, but that's a rare-ish case so not a big deal
                return valibotToJsonSchema(input);
            }
            const parent = valibotToJsonSchema(v.object({ child: input }), {
                errorMode: 'ignore',
            });
            const child = parent.properties.child;
            return parent.required?.length === 0 ? Object.assign(child, { optional: true }) : child;
        },
        effect: (input) => {
            const effect = dependencies.effect || disguisedEval(`require('effect')`);
            if (!effect) {
                throw new Error(`effect dependency could not be found - try installing it and re-running`);
            }
            if (!effect.Schema.isSchema(input)) {
                const message = `input was not an effect schema - please use effect version 3.14.2 or higher. See https://github.com/mmkal/trpc-cli/pull/63`;
                throw new Error(message);
            }
            return effect.JSONSchema.make(input);
        },
    };
};
function getVendor(schema) {
    // note: don't check for typeof schema === 'object' because arktype schemas are functions (you call them directly instead of `.parse(...)`)
    return schema?.['~standard']?.vendor ?? null;
}
const jsonSchemaVendorNames = new Set(Object.keys(getJsonSchemaConverters({})));
function looksJsonSchemaable(value) {
    const vendor = getVendor(value);
    return !!vendor && jsonSchemaVendorNames.has(vendor);
}
function prepareArktypeType(type) {
    let innerType = type;
    while (innerType) {
        if (innerType?.in && innerType.in !== innerType) {
            innerType = innerType.in;
        }
        else {
            break;
        }
    }
    return innerType;
}
// #endregion vendor specific stuff

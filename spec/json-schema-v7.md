# JSON Schema Validation Specification — Draft 07

Validation rules derived from
[draft-handrews-json-schema-validation-01](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01).

## §6.1 — validation keywords for any instance type

> r[json-schema-v7.type.primitive]
> The `type` keyword with a single primitive type string (null, boolean, string, number, integer, array, object)
> MUST succeed when the instance matches that type and MUST fail otherwise.
> See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.integer-subset]
> The type `"integer"` MUST match any number value with a zero fractional part. An integer
> is considered a valid number, so a schema with `"type": "number"` MUST also accept integer values.
> See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.array]
> TODO. See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.enum]
> TODO. See [§6.1.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.2).

> r[json-schema-v7.type.const]
> TODO. See [§6.1.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.3).

## §6.2 — validation keywords for numeric instances

> r[json-schema-v7.number.multiple-of]
> TODO. See [§6.2.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.1).

> r[json-schema-v7.number.maximum]
> TODO. See [§6.2.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.2).

> r[json-schema-v7.number.exclusive-maximum]
> TODO. See [§6.2.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.3).

> r[json-schema-v7.number.minimum]
> TODO. See [§6.2.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.4).

> r[json-schema-v7.number.exclusive-minimum]
> TODO. See [§6.2.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.5).

## §6.3 — validation keywords for string instances

> r[json-schema-v7.string.max-length]
> TODO. See [§6.3.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.1).

> r[json-schema-v7.string.min-length]
> TODO. See [§6.3.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.2).

> r[json-schema-v7.string.pattern]
> TODO. See [§6.3.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.3).

## §6.4 — validation keywords for array instances

> r[json-schema-v7.array.items-schema]
> TODO. See [§6.4.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.1).

> r[json-schema-v7.array.items-tuple]
> TODO. See [§6.4.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.1).

> r[json-schema-v7.array.additional-items]
> TODO. See [§6.4.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.2).

> r[json-schema-v7.array.max-items]
> TODO. See [§6.4.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.3).

> r[json-schema-v7.array.min-items]
> TODO. See [§6.4.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.4).

> r[json-schema-v7.array.unique-items]
> TODO. See [§6.4.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.5).

> r[json-schema-v7.array.contains]
> TODO. See [§6.4.6](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.6).

## §6.5 — validation keywords for object instances

> r[json-schema-v7.object.max-properties]
> TODO. See [§6.5.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.1).

> r[json-schema-v7.object.min-properties]
> TODO. See [§6.5.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.2).

> r[json-schema-v7.object.required]
> TODO. See [§6.5.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.3).

> r[json-schema-v7.object.properties]
> TODO. See [§6.5.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.4).

> r[json-schema-v7.object.pattern-properties]
> TODO. See [§6.5.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.5).

> r[json-schema-v7.object.additional-properties]
> TODO. See [§6.5.6](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.6).

> r[json-schema-v7.object.dependencies]
> TODO. See [§6.5.7](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.7).

> r[json-schema-v7.object.property-names]
> TODO. See [§6.5.8](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.8).

## §6.6 — keywords for applying subschemas conditionally

> r[json-schema-v7.conditional.if]
> TODO. See [§6.6.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.1).

> r[json-schema-v7.conditional.then]
> TODO. See [§6.6.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.2).

> r[json-schema-v7.conditional.else]
> TODO. See [§6.6.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.3).

## §6.7 — keywords for applying subschemas with boolean logic

> r[json-schema-v7.logic.all-of]
> TODO. See [§6.7.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.1).

> r[json-schema-v7.logic.any-of]
> TODO. See [§6.7.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.2).

> r[json-schema-v7.logic.one-of]
> TODO. See [§6.7.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.3).

> r[json-schema-v7.logic.not]
> TODO. See [§6.7.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.4).

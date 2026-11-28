# JSON Schema Validation Specification — Draft 07

Validation rules derived from
[draft-handrews-json-schema-validation-01](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01).

## §4.3.1 — JSON Schema values and keywords

> r[json-schema-v7.schema.root]
> A JSON Schema MUST be an object or a boolean.
> See [§4.3.1](https://json-schema.org/draft-07/draft-handrews-json-schema-01#rfc.section.4.3.1).

> r[json-schema-v7.schema.boolean-true]
> true: Always passes validation, as if the empty schema {}
> See [§4.3.1](https://json-schema.org/draft-07/draft-handrews-json-schema-01#rfc.section.4.3.1).

> r[json-schema-v7.schema.boolean-false]
> false: Always fails validation, as if the schema { "not":{} }
> See [§4.3.1](https://json-schema.org/draft-07/draft-handrews-json-schema-01#rfc.section.4.3.1).

## §6.1 — validation keywords for any instance type

> r[json-schema-v7.type.primitive]
> String values MUST be one of the six primitive types ("null", "boolean", "object", "array",
> "number", or "string"), or "integer" which matches any number with a zero fractional part.
>
> An instance validates if and only if the instance is in any of the sets listed for this keyword.
> See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.integer-subset]
> ... or "integer" which matches any number with a zero fractional part.
> See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.array]
> The value of this keyword MUST be either a string or an array. If it is an array, elements of the
> array MUST be strings and MUST be unique.
> See [§6.1.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.1).

> r[json-schema-v7.type.enum]
> The value of this keyword MUST be an array. This array SHOULD have at least one element. Elements
> in the array SHOULD be unique.
>
> An instance validates successfully against this keyword if its value is equal to one of the
> elements in this keyword's array value.
>
> Elements in the array might be of any value, including null.
> See [§6.1.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.2).

> r[json-schema-v7.type.const]
> The value of this keyword MAY be of any type, including null.
>
> An instance validates successfully against this keyword if its value is equal to the value of the
> keyword.
> See [§6.1.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.1.3).

## §6.2 — validation keywords for numeric instances

> r[json-schema-v7.number.multiple-of]
> The value of "multipleOf" MUST be a number, strictly greater than 0.
>
> A numeric instance is valid only if division by this keyword's value results in an integer.
> See [§6.2.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.1).

> r[json-schema-v7.number.maximum]
> The value of "maximum" MUST be a number, representing an inclusive upper limit for a numeric
> instance.
>
> If the instance is a number, then this keyword validates only if the instance is less than or
> exactly equal to "maximum".
> See [§6.2.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.2).

> r[json-schema-v7.number.exclusive-maximum]
> The value of "exclusiveMaximum" MUST be number, representing an exclusive upper limit for a
> numeric instance.
>
> If the instance is a number, then the instance is valid only if it has a value strictly less than
> (not equal to) "exclusiveMaximum".
> See [§6.2.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.3).

> r[json-schema-v7.number.minimum]
> The value of "minimum" MUST be a number, representing an inclusive lower limit for a numeric
> instance.
>
> If the instance is a number, then this keyword validates only if the instance is greater than or
> exactly equal to "minimum".
> See [§6.2.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.4).

> r[json-schema-v7.number.exclusive-minimum]
> The value of "exclusiveMinimum" MUST be number, representing an exclusive lower limit for a
> numeric instance.
>
> If the instance is a number, then the instance is valid only if it has a value strictly greater
> than (not equal to) "exclusiveMinimum".
> See [§6.2.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.2.5).

## §6.3 — validation keywords for string instances

> r[json-schema-v7.string.max-length]
> The value of this keyword MUST be a non-negative integer.
>
> A string instance is valid against this keyword if its length is less than, or equal to, the value
> of this keyword.
>
> The length of a string instance is defined as the number of its characters as defined by RFC 7159
> [RFC7159].
> See [§6.3.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.1).

> r[json-schema-v7.string.min-length]
> The value of this keyword MUST be a non-negative integer.
>
> A string instance is valid against this keyword if its length is greater than, or equal to, the
> value of this keyword.
>
> The length of a string instance is defined as the number of its characters as defined by RFC 7159
> [RFC7159].
>
> Omitting this keyword has the same behavior as a value of 0.
> See [§6.3.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.2).

> r[json-schema-v7.string.pattern]
> The value of this keyword MUST be a string. This string SHOULD be a valid regular expression,
> according to the ECMA 262 regular expression dialect.
>
> A string instance is considered valid if the regular expression matches the instance successfully.
> Recall: regular expressions are not implicitly anchored.
> See [§6.3.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.3.3).

## §6.4 — validation keywords for array instances

> r[json-schema-v7.array.items-schema]
> The value of "items" MUST be either a valid JSON Schema or an array of valid JSON Schemas.
>
> This keyword determines how child instances validate for arrays, and does not directly validate
> the immediate instance itself.
>
> If "items" is a schema, validation succeeds if all elements in the array successfully validate
> against that schema.
>
> Omitting this keyword has the same behavior as an empty schema.
> See [§6.4.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.1).

> r[json-schema-v7.array.items-tuple]
> The value of "items" MUST be either a valid JSON Schema or an array of valid JSON Schemas.
>
> This keyword determines how child instances validate for arrays, and does not directly validate
> the immediate instance itself.
>
> If "items" is an array of schemas, validation succeeds if each element of the instance validates
> against the schema at the same position, if any.
>
> Omitting this keyword has the same behavior as an empty schema.
> See [§6.4.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.1).

> r[json-schema-v7.array.additional-items]
> The value of "additionalItems" MUST be a valid JSON Schema.
>
> This keyword determines how child instances validate for arrays, and does not directly validate
> the immediate instance itself.
>
> If "items" is an array of schemas, validation succeeds if every instance element at a position
> greater than the size of "items" validates against "additionalItems".
>
> Otherwise, "additionalItems" MUST be ignored, as the "items" schema (possibly the default value of
> an empty schema) is applied to all elements.
>
> Omitting this keyword has the same behavior as an empty schema.
> See [§6.4.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.2).

> r[json-schema-v7.array.max-items]
> The value of this keyword MUST be a non-negative integer.
>
> An array instance is valid against "maxItems" if its size is less than, or equal to, the value of
> this keyword.
> See [§6.4.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.3).

> r[json-schema-v7.array.min-items]
> The value of this keyword MUST be a non-negative integer.
>
> An array instance is valid against "minItems" if its size is greater than, or equal to, the value
> of this keyword.
>
> Omitting this keyword has the same behavior as a value of 0.
> See [§6.4.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.4).

> r[json-schema-v7.array.unique-items]
> The value of this keyword MUST be a boolean.
>
> If this keyword has boolean value false, the instance validates successfully. If it has boolean
> value true, the instance validates successfully if all of its elements are unique.
>
> Omitting this keyword has the same behavior as a value of false.
> See [§6.4.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.5).

> r[json-schema-v7.array.contains]
> The value of this keyword MUST be a valid JSON Schema.
>
> An array instance is valid against "contains" if at least one of its elements is valid against the
> given schema.
> See [§6.4.6](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.4.6).

## §6.5 — validation keywords for object instances

> r[json-schema-v7.object.max-properties]
> The value of this keyword MUST be a non-negative integer.
>
> An object instance is valid against "maxProperties" if its number of properties is less than, or
> equal to, the value of this keyword.
> See [§6.5.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.1).

> r[json-schema-v7.object.min-properties]
> The value of this keyword MUST be a non-negative integer.
>
> An object instance is valid against "minProperties" if its number of properties is greater than,
> or equal to, the value of this keyword.
>
> Omitting this keyword has the same behavior as a value of 0.
> See [§6.5.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.2).

> r[json-schema-v7.object.required]
> The value of this keyword MUST be an array. Elements of this array, if any, MUST be strings, and
> MUST be unique.
>
> An object instance is valid against this keyword if every item in the array is the name of a
> property in the instance.
>
> Omitting this keyword has the same behavior as an empty array.
> See [§6.5.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.3).

> r[json-schema-v7.object.properties]
> The value of "properties" MUST be an object. Each value of this object MUST be a valid JSON
> Schema.
>
> This keyword determines how child instances validate for objects, and does not directly validate
> the immediate instance itself.
>
> Validation succeeds if, for each name that appears in both the instance and as a name within this
> keyword's value, the child instance for that name successfully validates against the corresponding
> schema.
>
> Omitting this keyword has the same behavior as an empty object.
> See [§6.5.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.4).

> r[json-schema-v7.object.pattern-properties]
> The value of "patternProperties" MUST be an object. Each property name of this object SHOULD be a
> valid regular expression, according to the ECMA 262 regular expression dialect. Each property
> value of this object MUST be a valid JSON Schema.
>
> This keyword determines how child instances validate for objects, and does not directly validate
> the immediate instance itself. Validation of the primitive instance type against this keyword
> always succeeds.
>
> Validation succeeds if, for each instance name that matches any regular expressions that appear as
> a property name in this keyword's value, the child instance for that name successfully validates
> against each schema that corresponds to a matching regular expression.
>
> Omitting this keyword has the same behavior as an empty object.
> See [§6.5.5](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.5).

> r[json-schema-v7.object.additional-properties]
> The value of "additionalProperties" MUST be a valid JSON Schema.
>
> This keyword determines how child instances validate for objects, and does not directly validate
> the immediate instance itself.
>
> Validation with "additionalProperties" applies only to the child values of instance names that do
> not match any names in "properties", and do not match any regular expression in
> "patternProperties".
>
> For all such properties, validation succeeds if the child instance validates against the
> "additionalProperties" schema.
>
> Omitting this keyword has the same behavior as an empty schema.
> See [§6.5.6](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.6).

> r[json-schema-v7.object.dependencies]
> This keyword specifies rules that are evaluated if the instance is an object and contains a
> certain property.
>
> This keyword's value MUST be an object. Each property specifies a dependency. Each dependency
> value MUST be an array or a valid JSON Schema.
>
> If the dependency value is a subschema, and the dependency key is a property in the instance, the
> entire instance must validate against the dependency value.
>
> If the dependency value is an array, each element in the array, if any, MUST be a string, and MUST
> be unique. If the dependency key is a property in the instance, each of the items in the
> dependency value must be a property that exists in the instance.
>
> Omitting this keyword has the same behavior as an empty object.
> See [§6.5.7](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.7).

> r[json-schema-v7.object.property-names]
> The value of "propertyNames" MUST be a valid JSON Schema.
>
> If the instance is an object, this keyword validates if every property name in the instance
> validates against the provided schema. Note the property name that the schema is testing will
> always be a string.
>
> Omitting this keyword has the same behavior as an empty schema.
> See [§6.5.8](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.5.8).

## §6.6 — keywords for applying subschemas conditionally

> r[json-schema-v7.conditional.if]
> This keyword's value MUST be a valid JSON Schema.
>
> This validation outcome of this keyword's subschema has no direct effect on the overall validation
> result. Rather, it controls which of the "then" or "else" keywords are evaluated.
>
> Instances that successfully validate against this keyword's subschema MUST also be valid against
> the subschema value of the "then" keyword, if present.
>
> Instances that fail to validate against this keyword's subschema MUST also be valid against the
> subschema value of the "else" keyword, if present.
>
> If annotations [annotations] are being collected, they are collected from this keyword's subschema
> in the usual way, including when the keyword is present without either "then" or "else".
> See [§6.6.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.1).

> r[json-schema-v7.conditional.then]
> This keyword's value MUST be a valid JSON Schema.
>
> When "if" is present, and the instance successfully validates against its subschema, then
> valiation succeeds against this keyword if the instance also successfully validates against this
> keyword's subschema.
>
> This keyword has no effect when "if" is absent, or when the instance fails to validate against its
> subschema. Implementations MUST NOT evaluate the instance against this keyword, for either
> validation or annotation collection purposes, in such cases.
> See [§6.6.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.2).

> r[json-schema-v7.conditional.else]
> This keyword's value MUST be a valid JSON Schema.
>
> When "if" is present, and the instance fails to validate against its subschema, then valiation
> succeeds against this keyword if the instance successfully validates against this keyword's
> subschema.
>
> This keyword has no effect when "if" is absent, or when the instance successfully validates
> against its subschema. Implementations MUST NOT evaluate the instance against this keyword, for
> either validation or annotation collection purposes, in such cases.
> See [§6.6.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.6.3).

## §6.7 — keywords for applying subschemas with boolean logic

> r[json-schema-v7.logic.all-of]
> This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
> Schema.
>
> An instance validates successfully against this keyword if it validates successfully against all
> schemas defined by this keyword's value.
> See [§6.7.1](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.1).

> r[json-schema-v7.logic.any-of]
> This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
> Schema.
>
> An instance validates successfully against this keyword if it validates successfully against at
> least one schema defined by this keyword's value.
> See [§6.7.2](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.2).

> r[json-schema-v7.logic.one-of]
> This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON
> Schema.
>
> An instance validates successfully against this keyword if it validates successfully against
> exactly one schema defined by this keyword's value.
> See [§6.7.3](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.3).

> r[json-schema-v7.logic.not]
> This keyword's value MUST be a valid JSON Schema.
>
> An instance is valid against this keyword if it fails to validate successfully against the schema
> defined by this keyword.
> See [§6.7.4](https://json-schema.org/draft-07/draft-handrews-json-schema-validation-01#rfc.section.6.7.4).

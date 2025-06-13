# OpenAPI Compiler

Work in progress!

Input: openapi.yaml and set of paths / methods to be compiled
Output: well-formed structures for serde serialization / deserialization of
  request / response bodies and URL / Headers serialization.

## Motivation

I found that it is hard to find the compiler of openapi 3.x.x specifcation for Rust.
However, this is pet-project that hopefully will be useful for wider audience in future.

## Design idea

Compilation is split to several phases:

1. Parse openapi specification using serde (src/schema)
2. Compile parsed specification. On this stage all references are resolved and
   only required schema objects are collected.
3. Build model from compiled object. On this stage all objects are flattened and
   ready for the code generation. (in-progress)
4. Code generation. Initially I plan Rust code generator. However I'll leave
   open possibility for generation for other languages. (todo)

## Other similar projects

- [OpenAPITools / openapi-generator](https://github.com/OpenAPITools/openapi-generator)
  written in Java. Supports output in many languages.
- [paperclip-rs / paperclip](https://github.com/paperclip-rs/paperclip) Rust native
  tooling for openapi specifications.
- [x52dev / oas3-rs](https://github.com/x52dev/oas3-rs) Rust native openapi 3.1
  specifications.

## License

MIT

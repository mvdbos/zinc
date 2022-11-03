# The Zinc changelog

## Version 0.1.5-ING-13 (2022-05-30)

### Enabled `self` keyword

In Zinc version 0.1.5, `self` references are not supported despite `self` is reserved as a keyword. The implementation of
`self` references in Zinc are available from version 0.2.x. Thus, we backported this feature to Zinc 0.1.5, which provides
us the following enhancements:

- `Self`: type alias for the surrounding struct
- `self`: variable reference for the surrounding struct
- Method invocation: from `Struct::fun(self, vars)` to `self.fun(vars)`

### Enabled intermodule dependencies
Zinc version 0.1.5 does not support intermodule dependencies. Dependencies can only be used in the main module of the project.
With this update, we enable modules depending on other modules by topologically sorting source files based on their mod
statements before starting compilation. This works, as long as there are no cyclic dependencies. If there are, compilation
fails.

### Fixed `endif` memory bug

This update fixes a memory bug in the zinc VMs data stack. We observed that the data stack does not clean its assigned types
when it returns from a scope such as a function call or a conditional statement. If a new data type needs to be assigned
to the freed index in the stack, compilation fails. Below is an example code statement:

```rust
fn pollute_stack(z: u8) {
}

fn test_stack( ) {
    if true {
        let index: u16 = 22;
    } else {
        let index: i16 = 23;
    }
}

fn main() -> i8 {
    pollute_stack(8);
    test_stack();
    -1
}
```
When we run this example, the compiler fails with the error message:
```
[ERROR   zvm] type error: expected u16, got u8
                 at ./src/main.zn:12:5 (at testStack)
[ERROR   zvm] runtime error: type error: expected u16, got u8
[ERROR zargo] virtual machine failure: exit status: 1
```
The error is caused by the omitted type allocation in the data stack, which was assigned as type `u8` in `pollute_stack()`.
function. Ideally, within the scope of the `testStack()` function, this type should be cleaned and reassigned as `u16`.

Our fix provides a workaround to the problem, which enables value assignment only if old and new values are the same type and
does nothing otherwise.

### Fixed scope for `struct`

Zinc version 0.1.5 does not allow to have the same method names within the scope of different structs. With this update,
we add support for the same method name in separate structs, which enables us to implement methods of similar nature for
different structs.

### Added Blake2s gadget 
In ZKFlow, we use Blake2s as the hash function to perform Merkle tree operations. Zinc does not provide a gadget for Blake2s
despite the underlying cryptographic library, `franklin-crypto` has its implementation. Thus, we implemented the Blake2s gadget
ourselves on top of Zinc. Blake2s gadget expects the input preimage in bits and outputs the digest in bits. Using the auxiliary
`to_bits` method of Zinc, Blake2s gadget can be used as follows:

```rust
use std::crypto::blake2s;
use std::convert::to_bits;

const BLAKE2S_HASH_SIZE: u64 = 256;
const INT32_BITS:u8 = 32;

fn main(preimage: u32) -> [bool; BLAKE2S_HASH_SIZE] {
    let preimage_bits: [bool; INT32_BITS] = to_bits(preimage);
    blake2s(preimage_bits)
}
```

### Fixed bit endianness in Blake2s gadget 
In its original format, the hash digest of the `franklin_crypto` library does not match with the original Blake2 specification
and with the BouncyCastle library. Both the original spec and the BouncyCastle require a little-endian representation of
**bytes** within the hash computation. And the same is for the `franklin_crypto`.
However, on top of that, `franklin_crypto` also requires little-endian ordering of **bits within each byte** due to the
`UInt32` object type used in the implementation. `UInt32` is a representation of 32 Boolean objects as an unsigned integer,
where the least significant bit is located in the first place.

To overcome the mismatch between the `franklin_crypto` and the original spec, we added a function in our gadget,
`reverse_byte_bits()`, which reverses the bit order within every byte before and after hashing operation.

### Added Blake2s multi-input gadget
In ZKFlow, we use Blake2s hash to compute the hash of two concatenated messages, such as `Hash(nonce || serialized_component)`.
We designed the `blake2s_multi_input` gadget to eliminate additional concatenation operations in the circuit. The gadget
handles concatenation under the hood. `blake2s_mutli_input` expects exactly two preimages in bits as input.
The gadget can be used within the circuity as follows:

```rust
use std::crypto::blake2s_multi_input;
use std::convert::to_bits;

const BLAKE2S_HASH_SIZE: u64 = 256;
const INT32_BITS:u8 = 32;

fn main(preimage1: u32, preimage2:u32) -> [bool; BLAKE2S_HASH_SIZE] {

    let preimage_bits_1: [bool; INT32_BITS] = to_bits(preimage1);
    let preimage_bits_2: [bool; INT32_BITS] = to_bits(preimage2);
    
    blake2s_multi_input(preimage_bits_1, preimage_bits_2)
    
}
```

The computations is equivalent to concatenation of two preimages and helps to eliminate concatenation steps:

```rust
use std::crypto::blake2s;
use std::convert::to_bits;

const BLAKE2S_HASH_SIZE: u64 = 256;
const INT32_BITS:u8 = 32;

fn main(preimage1: u32, preimage2:u32) -> [bool; BLAKE2S_HASH_SIZE] {

    let preimage_bits_1: [bool; INT32_BITS] = to_bits(preimage1);
    let preimage_bits_2: [bool; INT32_BITS] = to_bits(preimage2);
    
    let mut preimage_bits_concatenated : [bool; 2 * INT32_BITS] = [false; 2 * INT32_BITS];
    preimage_bits_concatenated[0..INT32_BITS] = preimage_bits_1;
    preimage_bits_concatenated[INT32_BITS..(2* INT32_BITS)] = preimage_bits_2; 

    blake2s(preimage_bits_concatenated)
}
```

Similar to Blake2s gadget, the multi-input gadget also implements `reverse_byte_bits()` to assure correct ordering of the
bits within each byte. Please check Blake2s gadget to learn more about this operation.

## Version 0.1.5 (2020-04-07)

#### Language

- forbidden the division operator `/`, but implemented `std::ff::invert` for `field` inversion
- allowed casting to types with lesser bitlength (runtime error on overflow)
- added the bitwise operators `|`, `|=`, `^`, `^=`, `&`, `&=`, `<<`, `<<=`, `>>`, `>>=`, `~` (constant expressions only)
- added the binary (e.g. `0b101010`) and octal (e.g. `0o52`) integer literals
- implemented match exhaustiveness checking without the binding or wildcard pattern
- removed `static` statements for global variables (use `const` instead)
- limited `match` scrutinee expression to boolean and integer only, since it is impossible to destructure complex types for now
- reserved some keywords (see [Appendix C](https://zinc.matterlabs.dev/appendix/C-keywords.html) of the Zinc book)

#### Compiler

- fixed the bug with `!` while passing a non-builtin function call as a built-in one's argument
- fixed the bug with duplicate match expression branches

#### Overall

- added a wrapper directory to the release archives to prevent tar- and zip-bombs

## Version 0.1.4 (2020-03-05)

#### Language

- added the Schnorr signature verification to the standard library
- made enumerations strongly typed, not just groups of constants
- match scrutinee can be any expression again (including structure literals)
- implemented automatic loop bounds range type upcasting
- implemented arithmetic assignment operators (`+=`, `-=`, `*=`, `/=`, `%=`)
- allowed constant expressions as array sizes (both types and literals)
- field division (i.e. multiplication by inverted value)
- field comparison (treated as unsigned numbers)

#### Compiler

- implemented advanced errors with Rust-like formatting, hints, location pointers
- added constant overflow checking at compile-time
- the constant expression Euclidean division and remainder now work just like in VM

#### Virtual machine

- fixed 'unconstrained' variables in hash functions
- fixed constraint generation when the same variable is encountered multiple times in the same expression
- fixed some type errors
- optimized constraint generation for deterministic expressions

#### Overall

- added the Schnorr signature tool

## Version 0.1.3 (2020-02-17)

#### Compiler

- fixed the compile error with a comment at the end of a file
- added an empty statement to allow optional semicolons

## Version 0.1.2 (2020-02-14)

#### Language

- the structure literal does not require the `struct` keyword anymore
- `dbg!(...)` string interpolation, e.g. `dbg!("{} + {} = {}", 2, 2, 4)`;
- `assert!(...)` now accepts an optional string message as the 2nd argument
- match scrutinee expression now can only be a single identifier (will be fixed soon)
- operators `/`, `%`, `>=`, `>`, `<=`, `<` are temporarily forbidden for the type `field`

#### Zargo

- the 'run' command now builds the circuit before running
- added the 'proof-check` command, which executes the sequence 'build + setup + proof + verify'
- circuit data (keys, inputs, outputs) moved from `build` to `data` folder

#### Compiler

- fixed many boundaries of integer types
- fixed the loop range overflow in some edge values of integer types
- fixed the invalid operand order bug in some cases
- fixed conflicting namespaces for functions and types with the same name
- improved some error messages

#### Virtual machine

- fixed `pedersen` hash
- fixed unsigned integers division
- fixed the `while` condition
- fixed the function argument order in some `std` functions
- made the `std::convert::from_bits_signed` result two-complement
- pretty error reporting with file, line, and column
- improved some error messages
- removed the redundant 'field' and 'value' keys from the structure type in input JSON templates

#### Overall

- full integration test coverage
- improved logging

## Version 0.1.1 (2020-02-08)

*Internal dogfooding/testing release*

## Version 0.1.0 (2020-01-31)

*Initial release*

# The file format standard - v1
The file format has no name. It's from remake. But it has a standard so that everyone can have their own implement.
## The Terms
"u8" means unsigned 8-bit integer. "i64" means signed 64-bit integer. 

All string was encoded as UTF-8.

## The Data Format
All numbers are little endian.

All strings are a struct. As:
```c
struct String{
    u64 StringLengthInByte;
    Array<u8> UTF8ByteConsequence;
}
```

## The File Format
The file starts with byte consequence [72 65 6d 61 6b 65]\(Hex\).

Then is a table that contain the data.

Content:
| Name | DateType |
|:----:|:--------:|
|    |     |



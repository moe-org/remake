# The file format standard - v1
The file format has no name. It's from remake. But it has a standard so that everyone can have their own implement.
## The Terms
"u8" means unsigned 8-bit integer. "i64" means signed 64-bit integer. 

All string was encoded as UTF-8.

A "structure"(or "struct") is as key word "struct" in c,c++,rust.

## The Data Format
All numbers are **little endian**.

A boolean(or bool) means "u8". When its value is 0,it is false. Otherwise it is true.

All strings are a struct. As:
```c
struct String{
    u64 StringLengthInByte;
    u8 UTF8Byte1;
    u8 UTF8Byte2;
    u8 UTF8Byte3;
    ...
}
```

A array contains a length(u64) field and data.
As:
```c
struct Array<T>{
    u64 ArrayItemCount;
    T item1;
    T item2;
    T item3;
    ...
}
```

A example:
```c
struct Array<String>{
    u64 HowManyStringsWeHave;
    String item1;
    String item2;
    String item3;
    ...
}
```

A map:
```c
struct Map<KEY,VALUE>{
    u64 MapPairCount;

    KEY KeyOfPair1;
    VALUE ValueOfPair1;

    KEY KeyOfPair2;
    VALUE ValueOfPair2;
    ...
}
```

Let's go.

## The File Format
The file starts with byte consequence [72 65 6d 61 6b 65]\(Hex\), means "remake" in ASCII.

As:

Content:
| Name | DateType |
|:----:|:--------:|
| File-Header | (`remake` ASCII)Byte Sequence |
| Platform | u64 |
| Version | u64 |
| Targets | Array<Target> |


The target:
```c
struct Target{
    String name;
    Array<String> dependences;
    Array<Command> commands;
}
```

The command:
```c
struct Command{
    String ExecutablePath;
    Array<String> Arguments;
    bool IgnoreErrors;
    Map<String,String> EnvironmentVariables;
    String WorkingDirectory;
}
```

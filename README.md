# ToasterNES
Simple NES emulator written in Rust.

![image](https://github.com/user-attachments/assets/92c708cc-c575-47aa-8acd-70c59c53c6ea)

## Building & Running
- The Rust toolchain is required: https://www.rust-lang.org/tools/install

- Use Cargo to build and run, passing in the path to the ROM as argument:  
`cargo run --release <ROM path>`

- To play the game, use the keyboard controls:

    | Key | Button |
    | -------- | ------- |
    | WASD | Up, Left, Down, Right |
    | Q | Select |
    | E | Start |
    | L | A |
    | K | B |

## Mapper Support
"Mappers" represent different types of NES cartridges. 

Currently supported mappers:

| Mapper | Example Games |
| -------- | ------- |
| 0 | _Donkey Kong_, _Super Mario Bros._, _Ice Climber_, _Dig Dug_|
| 1 | _The Legend of Zelda_, _Tetris_, _Metroid_, _Dr. Mario_, _Ninja Gaiden_|
| 2 | _DuckTales_, _Mega Man_, _Castlevania_, _Metal Gear_ |
| 3 | _Gradius_, _Paperboy_, _Track & Field_ |

## To-do List
- Add audio
- Support more mappers
- Support non-volatile memory

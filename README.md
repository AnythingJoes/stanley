# Stanley
Atari 2600 Emulator in Rust

# TODO
- [ ] Finish implementing instructions
  * [ ] Handle wrap-around http://www.6502.org/tutorials/6502opcodes.html
  * [x] Handle added cycle across page boundary http://www.6502.org/tutorials/6502opcodes.html
  * [ ] Decimal mode for ADC and SBC [more info](http://www.6502.org/tutorials/decimal_mode.html)
- [ ] Implement riot
    - [x] Implement riot timers
    - [x] Implement riot ram
    - [x] Implement riot player 1 input
    - [ ] Implement riot player 2 input
- [ ] Implement tia
    - [x] Implement WSYNC
    - [x] Figure out vblank
- [x] Add graphics
    - [x] Fix graphics
- [x] Figure out disassembly -- the dasm symbol file give very little information. Look into alternate assemblers, modifying dasm, or otherwise outputting a symbol file that will tell us things like: data segment location, code segment location, byte declaration info, differentiates between symbols with the same value, etc
- [ ] Use RGB colors instead of RGBA, and figure out color ording issue
- [ ] Add sound
- [ ] Handle bank switching cartridges
- [ ] Handle various other peripherals and variations

## HEXvaders

⚠️ **I'm rewriting this game using a helper UI library that I desinged. If you ask why, please read this [blog post](https://pky.me/blog/programming-is-modeling/) I wrote.**

As soon as you see an invader, convert its hexadecimal value to binary and flip the bits using numbers 1 through 8 on your keyboard. Once the bit pattern matches the hex value of an invader, it gets shot. Remember, don't let the invaders reach the shooter line!

This is the terminal implementation of [Flippy Bit And The Attack Of The Hexadecimals From Base 16](https://flippybitandtheattackofthehexadecimalsfrombase16.com).

### Demo

![Demo](demo.gif)

### Todo

- [x] Base game
- [x] Restart logic
- [ ] Keep track of the highest score
- [ ] Dynamically speed up the game at certain score points
- [ ] Move all hard-coded ASCII characters and magic numbers to `assets.rs`

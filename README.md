# Boxing in 4D

https://user-images.githubusercontent.com/6061310/209049817-bef56a86-1081-4037-95eb-0d9f3b037290.mp4

[Try it out](https://beaumayns.github.io/boxing_in_4d)

## Features

- Commits are (more or less) "step by step" guide
- No weird kinds of shaders, works in WebGL
- Collision detection with 4D [Minkowski portal refinement](http://xenocollide.snethen.com/mpr2d.html)
- 4D rigid-body physics
- Sequential Impulse solving for contact and joint constraints

## Left Undone

- Some kind of attitude indicator would be nice
- Other render methods - shadow projection instead of slicing?
- Needs more shapes - possibly Dual Contouring of SDF + convex decomposition

## Building

- `cargo run`
- [trunk](https://trunkrs.dev) `serve` to run the web version

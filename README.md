# Box 4D

Interact with a 3D slice of a 4D world, with 10 degrees of freedom movement and rigid body physics.

**[Try it out](https://beaumayns.github.io/box4d)**

https://user-images.githubusercontent.com/6061310/209049817-bef56a86-1081-4037-95eb-0d9f3b037290.mp4

## Features

- Commits are (more or less) a "step by step" guide
- No weird kinds of shaders, works in WebGL
- Collision detection with 4D [Minkowski portal refinement](http://xenocollide.snethen.com/mpr2d.html)
- 4D rigid-body physics
- Sequential Impulse solving for contact and joint constraints

## Left Undone

- Some kind of position and attitude indicator would be nice
- Other render methods - shadow projection instead of slicing?
- Needs more shapes - possibly Dual Contouring of SDF + convex decomposition

## Building

- `cargo run`
- [trunk](https://trunkrs.dev) `serve` to run the web version

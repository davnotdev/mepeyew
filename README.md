# Mepeyew

Computer graphics has gotten to the point where you can't just draw pixels onto
the screen anymore.
Rather, rendering APIs are used for drawing in order to efficiently utilize the hardware.
However, each platform has its own preferred APIs: DirectX on Windows, Metal on
MacOS, etc.
Each of these APIs have unique differences and quirks, so modern renderers are
designed to abstract to enforce equal support among APIs.
`mepeyew` is the rendering API abstraction for Rust created for [`mewo`](https://github.com/davnotdev/mewo),
designed with both modern and older rendering APIs in mind.
This allows `mepeyew` to be support constrained APIs like WebGL while still
retaining the power and control of APIs like Vulkan.

## Usage

> See the examples!

## To Be Done

> This project is a work in progress!

- Headless Rendering
- Advanced Configurations
- Shader Attachments
- Textures
- Compute
- Other API support

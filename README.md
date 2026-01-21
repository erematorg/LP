# ![Life's Progress Logo](/logo.png)

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![CI](https://github.com/erematorg/LP/workflows/CI/badge.svg)](https://github.com/erematorg/LP/actions)
[![Discord](https://img.shields.io/discord/1177787606432489564.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/u2J25aGy8c)

A systemic 2D platformer in progress with an evolving simulation framework at its core.

---

## What is LP?

*Life's Progress (LP)* is a systemic 2D platformer in progress with an evolving simulation framework at its core. Unlike traditional games, LP aims to build life and environments from first principles - atoms form molecules, molecules form cells, and cells evolve into organisms. The world evolves dynamically through physics and chemistry, with systems implemented incrementally and validated over time. Over time, parts of the framework may be reusable as standalone tools, but LP remains a game first.

Players will witness the rise and fall of species, the balance between organisms, and the subtle interdependencies that drive the progression of life in a dynamic world. Rather than controlling the world, players experience it from within, potentially inhabiting different creatures to explore survival from various perspectives.

## Status

LP is early-stage and in active development. The simulation crates are active and evolving; gameplay systems are still forming, APIs can change without notice, and some systems are prototypes or partial implementations.

## Design Goals

LP is built on several foundational principles that set it apart:

- **True Emergence**: Life and behaviors arise naturally from basic physical and chemical systems rather than being scripted
- **Unified Simulation (in progress)**: Systems are designed to interoperate in one framework, but not all subsystems are fully wired yet
- **Perspective-Based Experience**: Players experience the world from within, rather than playing as omnipotent manipulators
- **Persistent Consequences**: Actions have lasting impacts on a world that evolves organically over time
- **Scientific Authenticity**: Based on real principles of biology, ecology, and physics, simplified only where necessary

## Simulation Scope

LP is a real-time simulation framework for games that may grow into a standalone toolkit, grounded in experimentally verified physical laws and SI units, with LP-0 approximations for real-time performance.

- **Physics laws**: Newton, Coulomb, Fourier (and related IRL relations) are the basis for forces and transfers.
- **Explicit approximations**: LP-0 uses pairwise interactions, cutoffs, softening, and explicit integration for performance, all documented in code and docs.
- **Scope & roadmap**: LP-0 is not a continuum or quantum solver; higher-fidelity grid PDEs and MPM methods are planned.
- **Honesty goal**: Scientifically honest mechanics where shortcuts are explicit and gameplay abstractions are not presented as physics.

## Features

- **Realistic Ecosystem Simulation**: An immersive environment that evolves based on population dynamics, climate factors, and terrain deformations.
- **Open-World Exploration**: Traverse a vast 2D world filled with flora, fauna, and dynamic terrains, all interacting naturally.
- **Educational Gameplay**: Gain insights into how ecosystems work while enjoying engaging, emergent gameplay.
- **Bottom-Up Dynamics**: Matter and energy systems that build organically from fundamental forces to complex organisms.
- **Multi-Scale Experience**: Explore life from microscopic to macroscopic levels, each with unique survival challenges.

## Docs

For detailed information about *Life's Progress*, including gameplay mechanics, controls, and insights into the underlying ecosystem dynamics, check out our [official wiki](https://wiki.lifesprogress.com/). The documentation will guide you through the game's complex systems and help you get the most out of your experience.

## Community

Before participating in discussions with the community, you should familiarize yourself with our [Code of Conduct](.github/code_of_conduct.md).

- [**Discord**](https://discord.gg/u2J25aGy8c): Join our growing community to chat with other players and contributors, share ideas, and stay updated on the latest developments.

### Contributing

If you'd like to help build LP, check out the [Contributing Guidelines](.github/contributing.md).

We welcome contributions from the community in several ways:

- [Report bugs](https://github.com/erematorg/LP/issues/new?template=bug_report.yml)
- [Suggest new features](https://github.com/erematorg/LP/issues/new?template=feature_request.yml)
- [Browse existing issues](https://github.com/erematorg/LP/issues)

## License

LP is dual-licensed under either:

- MIT License (`LICENSE-MIT` or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (`LICENSE-APACHE` or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

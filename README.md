# SCRAPP---Screen-capturing-app
Screen recording system built with Rust and ASP.NET Core, featuring audio capture, video encoding, OAuth2 authentication, and authenticated video uploads to a web platform.

![Status](https://img.shields.io/badge/status-alpha-orange)
![Release](https://img.shields.io/badge/release-v0.1.0--alpha-blue)

> **⚠️ Development Status: Alpha**
>
> ScreenCap is an ongoing development project currently in its alpha stage. Core functionality has been implemented, but the system is not yet production-ready. Features, APIs, and implementation details may change as development continues.

SCRAPP is a desktop-and-web system developed as a project focused on
implementing an it system for supporting online tutoring. It is
designed to let users record their screens and audio on a desktop
computer, save recordings locally, and upload them to a web platform
where they can be managed, shared and played back.

The project consists of two separate applications that communicate over
HTTPS: a Rust desktop recorder and an ASP.NET Core web application.

## Project capabilities

### Desktop application

-   Screen capture and audio recording.
-   Video and audio capture and processing for recording sessions.
-   Local recording storage.
-   User authentication through an OAuth 2.0 / OpenID Connect flow.
-   Authenticated uploads to the web application.
-   Multiple upload strategies: streaming, multipart, and chunked
    uploads.
-   Upload-session management, including progress/status handling and
    completion.

### Web application

-   User accounts and authentication.
-   Authenticated API endpoints for desktop uploads.
-   Video and upload records associated with their owners.
-   Server-side storage of uploaded recordings.
-   A web interface for browsing, managing, and watching videos.
-   A video playback page that streams stored recordings.

## Technology stack

  -----------------------------------------------------------------------
  Component                           Technologies
  ----------------------------------- -----------------------------------
  Desktop application                 Rust, Slint

  Screen and audio capture            `scap`, CPAL

  Encoding and media processing       FFmpeg / `ffmpeg-next`, `rubato`

  Web application                     ASP.NET Core 8, Razor Pages

  Authentication and authorization    ASP.NET Core Identity, OpenIddict,
                                      OAuth 2.0 / OpenID Connect

  Data access                         Entity Framework Core, SQL Server
                                      LocalDB

  Front-end styling                   Tailwind CSS v4
  -----------------------------------------------------------------------

## Architecture overview

The desktop and web applications are developed as separate components.
The desktop application captures and encodes recordings, stores them
locally, and sends them to the web application using authenticated HTTPS
requests. The web application validates access tokens, associates
uploads with the authenticated user, stores the uploaded files and
related database records, and provides web-based video management and
playback.

The upload implementation supports different transfer strategies so
their behaviour and performance can be compared when handling large
files.

## Development status

**Status: Alpha — Work in Progress.**

ScreenCap is an actively developed project released in an early, functional state. Its main desktop-to-web workflow has been implemented, including screen and audio recording, local storage, authentication, authenticated video uploads, and web-based video access.

The project is not yet production-ready. Some components remain incomplete, including the full video-processing pipeline, which is currently represented by a basic implementation that simulates processing. Further testing, deployment improvements, and hardening are required before the system can be considered suitable for production use.

Expect changes to features, APIs, and internal implementation as development progresses.

## Repository structure

- **Thesis** - engineering thesis documenting the system's architecture, implementation, and design decisions.
- **Desktop_App_Latest_Release_Windows_X86_64** - Windows x86-64 pre-release build of the desktop application (.exe) for testing and demonstration purposes.

The project is split into two main applications:

-   **Desktop application** - Rust-based recording client.
-   **Web application** - ASP.NET Core web platform and upload API.

Refer to the individual application directories and their configuration
files for component-specific setup instructions.

## Notes

This repository is published as a software project and demonstration of
an integrated desktop-and-web workflow for recording, uploading, and
accessing video materials.

## Copyright and Usage Restrictions

Copyright © 2026 Radoslaw Madzia. All rights reserved.

This repository is publicly available for viewing and evaluation purposes only. It is intended to showcase an engineering thesis project and its implementation.

No permission or license is granted to use, copy, reproduce, modify, distribute, sublicense, or otherwise exploit any part of this repository, including its source code, documentation, thesis, and compiled binaries, without prior written permission from the copyright holder.

Both commercial and non-commercial use are prohibited without explicit authorization.

Making this repository publicly accessible does not grant any rights to its contents. All rights are reserved by the copyright holder.

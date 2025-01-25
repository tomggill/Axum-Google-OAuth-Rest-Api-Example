# Lift-Rust

Note: This project is currently WIP.

I want to thank [thanipro](https://github.com/thanipro) for his example [repository](https://github.com/thanipro/Axum-Rust-Rest-Api-Template) which helped greatly.

This project is an open source template for a rust Axum project with Google OAuth authentication setup.

### Introduction

In my effort to learn rust I found there isn't many examples of a full E2E authentication setup using rust Axum for the Google OAuth flow. I have created this project as an example for any future developers wanting to integrate with Google OAuth.

This repository includes:

- Google OAuth integration with auth middleware to validate and refresh access tokens.
- Repository / Service Layer separation.
- Logging.
- A testing setup that can be built upon.
- Git workflow to build & test the application automatically.

### Setup

1. Clone the project.
2. Rename `.env.example` to `.env` and populate with your DB and Google OAuth credentials:
   - To setup your Google OAuth client See [here](https://support.google.com/cloud/answer/6158849?hl=en).
3. Install `sqlx-cli` and run `sqlx migrate run`.
4. Run `cargo build` and then `cargo run`.

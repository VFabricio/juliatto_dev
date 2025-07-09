# juliatto.dev Instructions

## Introduction

This is the repository for my personal website.
It contains a blog as well as a home containing further information about myself.

## Technology Overview

At the center of juliatto.dev we have this codebase.
It is responsible for serving webpages, receiving newsletter subscriptions and sending out newsletter emails.
Resend is used for sending emails to our subscribers.

## Binaries

The application consists of three binaries organized in a Cargo workspace.

### server
This is the HTTP server.
It serves the website content, that is all static, and processes POST requests for newsletter subscriptions.
It also handles rate limiting.

### builder

This is responsible for building the Tera templates into the static content that will be served by `server`.
It runs in CI whenever new posts are added.

### sender

This is responsible for sending the newsletter to subscribers.
It uses Tera for HTML templating, just like the website itself.

## Code Architecture

All binaries should follow these architectural principles.
They use a streamlined hexagonal architecture.
In the core/ folder are the commands and queries that respond to business processes.
They should be just functions.
They must not perform any side effects directly.
Any kind of ambient interaction they require must be intermediated by port objects that they receive as arguments.
In the ports/ folder we have the implementations of such objects.
They are free to interact with databases, the filesystem and so on.
Each port should be implemented as a struct, with public methods exposing their functionality.
Connecting the core and ports, we have the adapters/ folder.
Each adapter is a Rust trait representing a set of functionalities exposed by a port.
Each port must then implement at least one such interface.
Beside the interfaces, the adapters folder should contain only auxiliary code, such as error enums that are returned by some method.

So, we use inversion of control to structure the application, but there is not IOC container.
That is, the injection of dependencies happens manually.
Some of the top level, infrastructure related components of the application, such as the web server are not abstracted and do not participate in component injection.
In `server` the composition roots are not centralized in the application root and instead exist within each HTTP request.
`builder` and `server`, on the other hand, assemble all components in their main function.

## Code Organization

We use rustfmt for code-formatting.
We use clippy for linting.
There is a pre-commit to hook to ensure that the code builds, is formatted correctly and approved by the linter before it can be pushed.

## Cloud Architecture

All the cloud architecture is managed via Terraform.
We use AWS for all of our cloud infrastructure.
The application runs directly on EC2 instances, via a Docker Swarm that we manage manually.
We use Traefik, also in EC2, as our load balancer.
It also handles certificate emission and TLS termination.
We use ECR for storing our container images.
We use RDS for the application database.

## CI/CD

We use Github Actions for CI/CD.
When the code is merged into the release branch in git a version tag must be created.
The tag is generated following semver.
We use conventional commits and so the commit messages must be inspected to know how the tag must be updated:
 - If all commit types are other than "feat" and there are no "breaking changes" paragraphs, the patch version must be updated.
 - If there are any "feat" commits, but no "breaking changes", the minor version must be updated.
 - If there are any "breaking changes", the major version should be updated.
The Docker image must then be built using the Dockerfile in the repository.
The image must use the tag we just generated in git.
The image is then pushed to ECR.

Notice that, at first, there is no automation for deploying the newly built images.
For the moment, we will SSH into EC2 and run Docker Swarm commands manually to pull in the new container versions.

## Database

Our database is a Postgresql instance running in RDS.
There is no need to avoid Postgresql specific functionality or restrict ourservel to ANSI SQL.
We must, however, take care to only use Postgresql features supported by RDS.
We use sqlx to intermediate all DB communication.
We use sqlx for database migrations.
All primary keys must be UUIDs.

## HTTP

We use axum for our HTTP server.
Static content is served from a directory.

## Observability

We use Open Telemetry extensively to monitor the project.
It is expected that, to a first approximation, every function call within our own code should be traced, along with the values of all parameters.
It may not be possible to follow this ideal 100%, due to excessive chattiness and costs in our observability infrastructure.
However, the approach must be to first instrument everything and only remove instrumentation when necessary.
We should use the `tracing` crate and related infrastructure for this.

## Error Handling

We use a mix of thiserror and anyhow for error handling.
Use thiserror for creating error enums when higher layers can be expected to handle the errors and pattern match on them.
Use anyhow for errors that cannot be reasonably handled (application needs to exit or return a 500 error).
Always add enough detail to errors to be captured by our observability infrastructure for notification and debugging.

## Configuration

Use environment variables in production.
For development convenience, configuration can be specified via a TOML file named `config.toml`.
Development configuration files must not be committed to version control as they may contain sensitive details.

## Serialization

We use serde for all serialization needs.

## Security

For now, no feature requires authentication or authorization.

## Local Development

Use docker compose to run a local instance of Postgresql for development.

## Environment Management

Development uses a configuration file (not committed to version control due to sensitive details).
Production uses environment variables passed to containers by Docker.

## Performance and Scalability

No specific performance or scalability targets currently.
We expect modest traffic and will optimize only if necessary.

## Monitoring

We run Jaeger in our infrastructure for OpenTelemetry trace collection.

## AI specific instructions

Do not use explanatory comments.
Run the check cargo check at the end of all interventions to make sure that nothing is broken.

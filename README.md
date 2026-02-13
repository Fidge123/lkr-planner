# LKR Planner

This application is a macOS application to make project planning easier.

## Features

### Project Synchronization

Status: Not implemented yet

The application synchronizes with Daylite CRM and Planradar.
It uses the [Daylite API](https://developer.daylite.app/reference/getting-started) and the [Planradar API](https://help.planradar.com/hc/en-gb/articles/15480453097373-Open-APIs).
Daylite projects are the source of truth for the project data.
The application will try to create Planradar projects automatically based on the appropriate template.

### Basic Employee Management

Status: Not implemented yet

You can maintain a list of employees with availability, skills and home location.
Each employee has an iCal calendar linked which is used to synchronize their assigned projects.
This configuration is stored in Daylite using the [Contacts API](https://developer.daylite.app/reference/contacts).

### Project Planning

Status: In progress

A weekly calendar view allows you to assign employees to Daylite projects.
This syncs with the iCal calendar of the employee.
Any assigned projects for the current week are also created/reopened in Planradar.

## Development

### Local Quality Workflow

Before committing changes, ensure all quality checks pass:

```bash
# Run tests
bun test

# Run tests in watch mode during development
bun test:watch

# Check code quality (lint)
bun lint

# Auto-fix linting issues
bun lint:fix

# Check code formatting
bun format:check

# Auto-format code
bun format
```

### Running the Application

```bash
# Development mode
bun tauri dev

# Build for macOS
bun build:macos
```

**Note:** The same quality checks (`bun test`, `bun lint`, `bun format:check`) are run in CI/CD, so running them locally ensures your changes will pass automated checks.


# {{ forge.str.pascal(forge.project.name) }}

{{ forge.str.pascal(forge.vars.frontend) }} frontend · {{ forge.str.pascal(forge.vars.backend) }} backend · {{ forge.vars.database }}

## Getting started

```bash
# Install frontend dependencies
cd frontend && {{ pm }} install

# Start the dev server
{{ pm }} run dev
```

## Stack

| Layer    | Technology                                                                         |
|----------|------------------------------------------------------------------------------------|
| Frontend | {{ forge.vars.frontend }}                                                          |
| Backend  | {{ forge.vars.backend }}                                                           |
| Database | {{ forge.vars.database }}                                                          |
| Ports    | {{ forge.vars.port_frontend }} (frontend) · {{ forge.vars.port_backend }} (backend) |

{{ if forge.vars.docker then return [[
## Docker

Start the full stack with Docker Compose:

```bash
docker compose up
```
]] end }}

## Project structure

```
{{ forge.project.name }}/
├── frontend/       # {{ forge.vars.frontend }} app
├── backend/        # {{ forge.vars.backend }} server
└── docker-compose.yml
```

---

_Scaffolded with [Forge](https://github.com/your-org/forge)_

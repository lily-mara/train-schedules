# Caltain Schedules

## Development

All development commands go through [earthly](https://earthly.dev). Outside of
deployment which requires the caprover CLI, there are no other dependencies.
Here are some helpful targets:

- `earthly +serve` - run a hot-reloading dev server for the frontend and backend on `localhost:8080`
- `earthly +serve-docker` - build the production docker image and run it in docker-compose on `localhost:8080`
- `earthly +deploy` - build the production docker image, push it to docker hub, deploy it using caprover

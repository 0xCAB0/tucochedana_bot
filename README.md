# Tu Coche Dana Bot

> [!IMPORTANT]
> This is an unoffical project that integrates with [tucochedana.es](https://tucochedana.es/) to allow users to find and report missing cars during the 2024 floods in Valencia.

## Commands

```text
start - Despliega el menú de opciones y el mensaje de bienvenida
add_vehicle_message - Registra la matrícula del vehículo que buscas
get_my_vehicles - Devuelve el listado de vehículos que has registrado
help - Muestra un mensaje de ayuda sobre cómo usar el bot
```

## How to run tests

- [x] Just need to create a postgres DB using the DATABASE_URL env variable -> `make db`
- [x] Run migrations using `make diesel-test`, just until we support testing_repo for Fang tasks

> [!NOTE]
> Original project by [@Betisman](https://github.com/Betisman/tucochedana-checker)
<div align="center">

<h1><b>Tu Coche Dana Bot</b></h1>

<a href="https://t.me/TuCocheDanaBot" alt="Checkout on Telegram">
        <img width=150 src="https://img.shields.io/badge/-telegram-red?color=white&logo=telegram&logoColor=black"/></a>

</div>

> [!IMPORTANT]
> This is an unoffical project that integrates with [tucochedana.es](https://tucochedana.es/) to allow users to find and report missing cars during the 2024 floods in Valencia.
## Commands


- **`start`**  
  Displays the menu of options and a welcome message.

- **`add_vehicle_message`**  
  Registers the license plate of the vehicle you are looking for.

- **`get_my_vehicles`**  
  Returns the list of vehicles you have registered.

- **`start_fetch`**  
  Activates the search for the saved vehicles.

- **`stop_fetch`**  
  Deactivates the search for the saved vehicles.

- **`help`**  
  Shows a help message about how to use the bot.


### Paste-bin to Telegram bot setup

```text
start - Despliega el menú de opciones y el mensaje de bienvenida
add_vehicle_message - Registra la matrícula del vehículo que buscas
get_my_vehicles - Devuelve el listado de vehículos que has registrado
start_fetch - Activa la búsqueda de los vehículos guardados
stop_fetch - Desactiva la búsqueda de los vehículos guardados
help - Muestra un mensaje de ayuda sobre cómo usar el bot
```

## Development

### Running locally

Setting up the SSL certificates and the proper security to run a webhook based bot can be a pain.

Checkout any of these articles for alternative methods to run the bot in your local machine:
  - [Setup webhooks locally]((https://www.bafonins.xyz/articles/telegram-bot-local-testing/#the-problem-with-setwebhook))
  - [tbot setup](https://gitlab.com/SnejUgal/tbot/-/wikis/How-to/How-to-use-webhooks#configuring-your-server)


### Running tests

- [x] Just need to create a postgres DB using the DATABASE_URL env variable -> `make db`
- [x] Run migrations using `make diesel-test`, just until we support testing_repo for Fang tasks

## Credits
> [!NOTE]
> Original project by [@Betisman](https://t.me/tucochedanachecker_bot)
> 
> Official [tu-coche-dana](https://tucochedana.es) project
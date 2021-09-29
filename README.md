# Tiny tickets

A very simple helpdesk system allowing to create and manage tickets from a web app and a native app.
Tickets are related to assets, users can create and close tickets, desk can create comments only.

## Architecture

### Back end

The backend is made with Rust and Rocket.
This backend makes use of SQLite.

### Front end

The front end is a flutter application, a web release is served by the back end, but a native app is also provided.

## Roles model

Roles haves the following rights :

| Role  | Assets | Tickets | Comments |
| ----- | ------ | ------- | -------- |
| Users | R      | CR      | CR       |
| Admin | CRUD   | CRUD    | CRUD     |

The rights are defined by tokens, set as environment variables.

## Environment variables

| Environment Variable | Usage                                                 | Default value                     |
| -------------------- | ----------------------------------------------------- | --------------------------------- |
| USER_TOKEN           | API token for users                                   | random value (printed at startup) |
| ADMIN_TOKEN          | API token for admins                                  | random value (printed at startup) |
| MAIL_SERVER          | hostname of mail server for mail notifications        | empty (mails will not be send)    |
| MAIL_USER            | mail user for authenticating on the mail server       | empty (mails will not be send)    |
| MAIL_PASSWORD        | mail password                                         | empty (mails will not be send)    |
| MAIL_FROM            | mail address the mails will be sent from              | empty (mails will not be send)    |
| TICKET_MAIL_TO       | mail address the mails on new ticket will be sent to  | empty (mails will not be send)    |
| COMMENT_MAIL_TO      | mail address the mails on new comment will be sent to | empty (mails will not be send)    |
| APP_TITLE            | title displayed on top of the app                     | Tiny Tickets                      |

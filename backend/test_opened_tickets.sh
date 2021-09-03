#!/bin/bash
curl 'http://localhost:8000/api/tickets/mail_open' -H 'X-TOKEN: $ADMIN$admin'

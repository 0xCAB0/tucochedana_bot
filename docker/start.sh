#!/usr/bin/env sh

echo "set db value :$SET_DB"

if [ ! -z $SET_DB ]; then
  echo "Setting the database"
  ./diesel migration run
fi

if [ ! -z $REVERT_DB ]; then
  echo "Revert to last migration"
  ./diesel migration revert
fi

echo "Running Bot $BOT_NAME"
./$BOT_NAME

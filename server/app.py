from __future__ import annotations

from flask import Flask, request, abort, jsonify
from flask_socketio import SocketIO
from pymongo import MongoClient
from typing import Dict
from datetime import datetime

#
# Initialize logger
#
import logging

logging.basicConfig(level=logging.INFO)
logging.info("logger initialized")

#
# Load configs.
#

import os

server_mode = os.environ.get("SITUPS_SERVER_MODE")

if server_mode == "production":
    from config.production import Config

    logging.info(
        f"running server in {server_mode} mode at {Config.HOST}:{Config.PORT}"
    )
else:
    from config.development import Config

    logging.info(
        f"running server in {server_mode} mode at {Config.HOST}:{Config.PORT}"
    )

#
# Initialize Flask.
#
app = Flask(__name__)
app.config.from_object(Config)

# TODO:
#   Go over correct procedures before running this in production.
socketio = SocketIO(app, cors_allowed_origins="*")

#
# Initialize database.
#
client = MongoClient(Config.DATABASE_URL)
db = client[Config.DATABASE_NAME]
logging.info("established connection with database")

#
# Setup routes.
#


@app.route("/users", methods=["GET"])
def route_users():
    return list(db["users"].find({}, {"userID": "$_id", "displayName": 1, "theme": 1}))


def try_int(x) -> int:
    if type(x) != int:
        raise TypeError(f"expected '{x}' to be int")
    return x


def try_str(x) -> str:
    if type(x) != str:
        raise TypeError(f"expected '{x} to be str")
    return x


class ScheduleDate:
    def __init__(self, year: int, month: int, day: int):
        self.year = year
        self.month = month
        self.day = day

    @staticmethod
    def from_json(obj: Dict) -> ScheduleDate:
        return ScheduleDate(
            try_int(obj["year"]),
            try_int(obj["month"]),
            try_int(obj["day"]),
        )

    def json(self):
        return {"year": self.year, "month": self.month, "day": self.day}


class EntryKey:
    def __init__(self, user_id: str, schedule_date: ScheduleDate):
        self.user_id = user_id
        self.schedule_date = schedule_date

    @staticmethod
    def from_json(obj: Dict) -> EntryKey:
        return EntryKey(
            try_str(obj["userID"]), ScheduleDate.from_json(obj["scheduleDate"])
        )

    def json(self) -> Dict:
        return {"userID": self.user_id, "scheduleDate": self.schedule_date.json()}


class EntryRequest:
    class UpdateAmount:
        def __init__(self, obj: Dict):
            self.entry_key = EntryKey.from_json(obj)
            self.amount = try_int(obj["amount"])

        def json(self):
            return {"entriesKey": self.entry_key.json(), "amount": self.amount}

    class Get:
        """
        Request sent by websocket containing a `EntryKey`. The server acknoledges the message
        with the available data for the entry with the given key.

        TODO:
          Implement robust strategy for requests where the query can't find an entry.
        """

        def __init__(self, json: Dict):
            self.entry_key = EntryKey.from_json(json["entryKey"])


@app.route("/entry/update-amount", methods=["POST"])
def route_update():
    try:
        req = EntryRequest.UpdateAmount(request.json)

        db["commits"].insert_one(
            {"date": datetime.utcnow(), "kind": "update-amount", "data": req.json()}
        )

        db["entries"].update_one(
            {"_id": req.entry_key.json()},
            {"$set": {"amount": req.amount}},
            upsert=True,
        )
        return jsonify(success=True)

    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)


@socketio.on("connect")
def socketio_connect():
    logging.info(f"connected with socket on namespace '/entry'")


@socketio.on("disconnect")
def socketio_disconnect():
    logging.info(f"disconnected with socket")


@socketio.on("acc-connect", namespace="/entry")
def socketio_entry_message(json):
    logging.info(f"entry/acc-connect: {json}")


@socketio.on("get", namespace="/entry")
def socketio_entry_get(json):
    logging.info(f"socket_entry_get: ${json}")

    try:
        req = EntryRequest.Get(json)

        if x := db["entries"].find_one(
            {"_id": req.entry_key.json(), "amount": {"$gt": 0}},
            {"_id": 0, "amount": 1},
        ):
            return {"amount": x["amount"]}
        else:
            return {"amount": None}

    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)


if __name__ == "__main__":
    socketio.run(app, host=Config.HOST, port=Config.PORT)

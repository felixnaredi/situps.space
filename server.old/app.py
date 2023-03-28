from __future__ import annotations

from flask import Flask, request, abort
from flask_socketio import SocketIO, emit
from pymongo import MongoClient
from typing import Dict, Optional
from datetime import datetime

# TODO:
#   This is just a temporary work around.
from engineio.payload import Payload
Payload.max_decode_packets = 128

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

    logging.info(f"running server in {server_mode} mode at {Config.HOST}:{Config.PORT}")
else:
    from config.development import Config

    logging.info(f"running server in {server_mode} mode at {Config.HOST}:{Config.PORT}")

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


@app.route("/api/users", methods=["GET"])
def route_users():
    return list(db["users"].find({}, {"userID": "$_id", "displayName": 1, "theme": 1}))


# TODO:
#   The error handling is sloppy. It should be clear when errors are thrown because the request was
#   bad and what went wrong.
#
#   VALUE: 4, ESTIMATE: 3

# TODO:
#   Should parsing commits allow extra fields (like it does now) or should it be strict?
#

# TODO:
#   The boilerplate needed to create database and event structues is causing too much bugs. The
#   debugging is hurting the velocity.
#
#   It would be worth a moderate to large amount of effort to lessen it. Third party libraries,
#   annotations and clever interspection are all welcome. They are only required to store data so it
#   should be possible.
#
#   Too be clear. Writting them goes fast and they are readable. It is the time debugging every 
#   minor change to them that is expensive.
#
#   VALUE: 7, ESTIMATE: 7


def try_int(x) -> int:
    if type(x) != int:
        raise TypeError(f"expected '{x}' to be `int`")
    return x

def try_optional_int(x) -> Optional[int]:
    if type(x) == int or x is None:
        return x
    else:
        raise TypeError(f"expected '{x}' to be `int` or `None`")


def try_str(x) -> str:
    if type(x) != str:
        raise TypeError(f"expected '{x} to be `str`")
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


class EntryData:
    def __init__(self, amount: Optional[int]):
        self.amount = amount

    @staticmethod
    def from_json(json: Dict) -> EntryData:
        return EntryData(amount=try_optional_int(json["amount"]))

    def json(self) -> Dict:
        return {"amount": self.amount}


def fetch_entry_data_from_database(entry_key: EntryKey) -> Optional[EntryData]:
    if x := db["entries"].find_one(
        {"_id": entry_key.json(), "amount": {"$gt": 0}},
        {"_id": 0, "amount": 1},
    ):
        return EntryData.from_json(x)
    else:
        return None


class EntryEvent:
    class Get:
        """
        Request sent by websocket containing a `EntryKey`. The server acknoledges the message
        with the available data for the entry with the given key.
        """

        def __init__(self, json: Dict):
            self.entry_key = EntryKey.from_json(json["entryKey"])

    class Update:
        """
        Request sent by websocket containing a `EntryKey` and data used to update the entry with
        the key.
        """

        def __init__(self, json: Dict):
            self.entry_key = EntryKey.from_json(json["entryKey"])
            self.new_value = EntryData.from_json(json["newValue"])

        def json(self):
            return {"entryKey": self.entry_key.json(), "newValue": self.new_value.json()}

    class StateChanged:
        """
        The response from the server broadcast to the clients when the state on the entries changes.
        """

        def __init__(
            self,
            entry_key: EntryKey,
            old_value: Optional[EntryData],
            new_value: EntryData,
        ):
            self.entry_key = entry_key
            self.old_value = old_value
            self.new_value = new_value

        def json(self):
            return {
                "entryKey": self.entry_key.json(),
                "oldValue": self.old_value.json() if self.old_value else None,
                "newValue": self.new_value.json(),
            }


@socketio.on("ack-connect", namespace="/entry")
def socketio_entry_ack_connect():
    logging.info(f"entry/ack-connect: {request.sid}")


@socketio.on("get", namespace="/entry")
def socketio_entry_get(json):
    logging.info(f"socket_entry_get: {json}")

    try:
        req = EntryEvent.Get(json)
    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)

    if data := fetch_entry_data_from_database(req.entry_key):
        return {"entryData": data.json()}
    else:
        return {"entryData": None}


@socketio.on("update", namespace="/entry")
def socketio_entry_update(json):
    logging.info(f"socket_entry_update: {json}")

    #
    # Parse request. Abort if it is bad.
    #
    try:
        req = EntryEvent.Update(json)
    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)

    #
    # Log commit in database.
    #
    db["commits"].insert_one(
        {
            "date": datetime.utcnow(),
            "client": request.sid,
            "namespace": "/entry",
            "event": "update",
            "data": req.json(),
        }
    )

    #
    # Fetch old value.
    #
    old_value = fetch_entry_data_from_database(req.entry_key)

    #
    # Update the entry, or add a new one if the key was not present.
    #
    db["entries"].update_one(
        {"_id": req.entry_key.json()},
        {"$set": req.new_value.json()},
        upsert=True,
    )

    #
    # Broadcast the update.
    #
    emit(
        "state-changed",
        EntryEvent.StateChanged(req.entry_key, old_value, req.new_value).json(),
        broadcast=True,
    )


if __name__ == "__main__":
    socketio.run(app, host=Config.HOST, port=Config.PORT)

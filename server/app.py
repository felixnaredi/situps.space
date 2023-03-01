from __future__ import annotations

from flask import Flask, request, abort, jsonify
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

if os.environ.get("SITUPS_SERVER_MODE") == "production":
    logging.info("running server in production mode")
    from config.production import Config
else:
    logging.info("running server in development mode")
    from config.development import Config

#
# Initialize Flask.
#
app = Flask(__name__)
app.config.from_object(Config)

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


class EntriesKey:
    def __init__(self, user_id: str, schedule_date: ScheduleDate):
        self.user_id = user_id
        self.schedule_date = schedule_date

    @staticmethod
    def from_json(obj: Dict) -> EntriesKey:
        return EntriesKey(
            try_str(obj["userID"]), ScheduleDate.from_json(obj["scheduleDate"])
        )

    def json(self) -> Dict:
        return {"userID": self.user_id, "scheduleDate": self.schedule_date.json()}


class AmountRequest:
    def __init__(self, obj: Dict):
        self.entries_key = EntriesKey.from_json(obj)


@app.route("/amount", methods=["POST"])
def route_amount():
    try:
        req = AmountRequest(request.json)

        if x := db["entries"].find_one(
            {"_id": req.entries_key.json(), "amount": {"$gt": 0}},
            {"_id": 0, "amount": 1},
        ):
            return x
        else:
            return {"amount": None}
    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)


class UpdateRequest:
    def __init__(self, obj: Dict):
        self.entries_key = EntriesKey.from_json(obj)
        self.amount = try_int(obj["amount"])

    def json(self):
        return {"entriesKey": self.entries_key.json(), "amount": self.amount}


@app.route("/update", methods=["POST"])
def route_update():
    try:
        req = UpdateRequest(request.json)

        db["commits"].insert_one(
            {"date": datetime.utcnow(), "kind": "update-amount", "data": req.json()}
        )

        db["entries"].update_one(
            {"_id": req.entries_key.json()},
            {"$set": {"amount": req.amount}},
            upsert=True,
        )

        return jsonify(success=True)
    except Exception as error:
        logging.error(f"{type(error)} - {error}")
        abort(400)

if __name__ == "__main__":
    app.run(host=Config.HOST, port=Config.PORT)

use("test-get-room-properties");
db.entries.aggregate([
  {
    $match: {
      "_id.date": {
        $in: [
          {
            year: 1555,
            month: 2,
            day: 16,
          },
          {
            year: 1555,
            month: 2,
            day: 15,
          },
          {
            year: 1555,
            month: 2,
            day: 14,
          },
        ],
      },
      "_id.room": ObjectId("6452c7ac9544b2371d5ffcf2"),
    },
  },
  {
    $group: {
      _id: "$_id.date",
      users: {
        $push: "$_id.user",
      },
      entries: {
        $push: {
          user: "$_id.user",
          amount: "$amount",
        },
      },
    },
  },
]);

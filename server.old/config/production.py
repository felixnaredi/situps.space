import config.local.production

class Config(config.local.production.Config):
    ENV = "production"
    DEBUG = False
    DATABASE_NAME = "production"
    HOST = "0.0.0.0"
    # DATABASE_URL = "mongodb://localhost:27017"
    # PORT = 5001

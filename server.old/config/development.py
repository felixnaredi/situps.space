import config.local.development

class Config(config.local.development.Config):
    ENV = "development"
    DEBUG = True
    DATABASE_NAME = "development"
    # DATABASE_URL = "mongodb://localhost:27017"
    # HOST = "127.0.0.1"
    # PORT = 5000

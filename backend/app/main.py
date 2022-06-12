from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from app.routers import users, tags
from tortoise import Tortoise
from config import settings


app = FastAPI()
app.include_router(users.router)
app.include_router(tags.router)


app.add_middleware(
    CORSMiddleware,
    allow_origins="*",
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.on_event("startup")
async def init():
    await Tortoise.init(
        db_url=settings["cockroachdb_dsn"],
        modules={"models": ["models"]},
    )
    # Generate the schema
    await Tortoise.generate_schemas()


@app.on_event("shutdown")
async def shutdown():
    await Tortoise.close_connections()

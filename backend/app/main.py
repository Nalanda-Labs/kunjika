from os import strerror
from typing import List
from fastapi import Depends, FastAPI, HTTPException, Request
from fastapi.responses import ORJSONResponse
import crud, schemas, security
from fastapi.security import OAuth2PasswordBearer, OAuth2PasswordRequestForm
from fastapi.middleware.cors import CORSMiddleware
from datetime import timedelta
from jwt import decodeJWT
from routers import files
import re


ACCESS_TOKEN_EXPIRE_MINUTES = 30


# models.Base.metadata.create_all(bind=engine)
oauth2_scheme = OAuth2PasswordBearer(tokenUrl="token")
app = FastAPI()

app.include_router(files.router)

origins = "*"

app.add_middleware(
    CORSMiddleware,
    allow_origins=origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

from tortoise import Tortoise


@app.on_event("startup")
async def init():
    # Here we create a SQLite DB using file "db.sqlite3"
    #  also specify the app name of "models"
    #  which contain models from "app.models"
    await Tortoise.init(
        db_url="postgres://shiv:yagyavalkya@localhost:26257/diskas",
        modules={"models": ["models"]},
    )
    # Generate the schema
    await Tortoise.generate_schemas()


# Dependency
# def get_db():
#     db = SessionLocal()
#     try:
#         yield db
#     finally:
#         db.close()


@app.post("/users/", response_model=schemas.User, response_class=ORJSONResponse)
async def create_user(user: schemas.UserCreate):
    print(user.email)
    regex = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"
    print(user.email)
    db_user = await crud.get_user_by_email(email=user.email)
    print(db_user)
    if re.fullmatch(regex, user.email) == None:
        raise HTTPException(status_code=409, detail="Invalid Email Address")
    if db_user:
        raise HTTPException(status_code=409, detail="Email already registered")
    return await crud.create_user(user=user)


@app.get("/users/", response_model=List[schemas.User])
def read_users(skip: int = 0, limit: int = 100):
    users = crud.get_users(skip=skip, limit=limit)
    return users


@app.get("/users/{user_id}", response_model=schemas.User)
def read_user(user_id: int):
    db_user = crud.get_user(user_id=user_id)
    if db_user is None:
        raise HTTPException(status_code=404, detail="User not found")
    return db_user

@app.post("/token", response_model=schemas.Token)
async def login_for_access_token(form_data: OAuth2PasswordRequestForm = Depends()):
    # user = authenticate_user(fake_users_db, form_data.username, form_data.password)
    # print(form_data.username)
    db_user = crud.get_user_by_email(email=form_data.username)
    if not (
        security.verify_hash(form_data.password, db_user.salt).decode("utf-8")
        == db_user.hashed_password
    ):
        raise HTTPException(
            status_code=401,
            detail="Incorrect username or password",
            headers={"WWW-Authenticate": "Bearer"},
        )
        # return {"message":"ERROR"}

    access_token_expires = timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    access_token = security.create_access_token(
        data={"sub": db_user.email}, expires_delta=access_token_expires
    )
    return {"access_token": access_token}

# Replace with JWT Access token response
@app.post("/login/", response_model=schemas.User)
def login(user: schemas.UserCreate):
    db_user = security.get_user_by_email(email=user.email)
    if (security.verify_hash(user.password,db_user.salt).decode('utf-8') == db_user.hashed_password):
        return db_user
    else:
        raise HTTPException(status_code=400, detail="Invalid Login")

# from sqlalchemy.orm import Session
import os
import hashlib
import base64
from datetime import datetime, timedelta
from typing import Optional, OrderedDict
from jwt import *
import json
from models import Users, Note, Item

import models, schemas

async def get_user(user_id: int):
    return await Users.filter(Users.id == user_id).first()

async def get_user_by_email(email: str):
    return await Users.filter(email = email).first()


async def get_users(skip: int = 0, limit: int = 100):
    return await Users.offset(skip).limit(limit).all()

async def create_user(user: schemas.UserCreate):
    salt = os.urandom(32)
    key = hashlib.pbkdf2_hmac(
        'sha256', # The hash digest algorithm for HMAC
        user.password.encode('utf-8'), # Convert the password to bytes
        salt, # Provide the salt
        100000 # It is recommended to use at least 100,000 iterations of SHA-256 
    )   
        #Bytes encoded to Base64 but still in byte format
    encodedSalt = base64.b64encode(salt)
    encodedKey = base64.b64encode(key)
    # fake_hashed_password = user.password + "notreallyhashed"
     
    db_user = Users(email=user.email, hashed_password=encodedKey.decode('utf-8'),salt=encodedSalt.decode('utf-8'))
    await db_user.save()
    return db_user



async def get_items(skip: int = 0, limit: int = 100):
    return Item.offset(skip).limit(limit).all()



async def create_user_item(item: schemas.ItemCreate, user_id: int):
    db_item = Item(**item.dict(), owner_id=user_id)
    await db_item.save()
    return db_item

async def create_user_note(note:str, user_id:int):
    db_item = Note(**note, owner_id=user_id)
    await db_item.save()
    return note

async def update_user_note(note:str,user_id:int):
    # print(note)
    # note = json.loads(note)
    db_item = Note(**note, owner_id=user_id)
    updateObject = await Note.filter_by(owner_id=user_id,id=note["id"]).first()
    updateObject.description = db_item.description
    updateObject.title = db_item.title
    await updateObject.save()
    return note

async def get_user_notes(user: schemas.User,skip: int = 0, limit: int = 100):
    items = Note.filter_by(owner_id=user.id).offset(skip).limit(limit).all()
    notes = []
    for item in items:
        note = OrderedDict()
        note["id"]=item.id
        note["title"] =  item.title
        note["description"] = item.description
        notes.append(note)
    return notes

async def delete_user_note(note:str,user_id: int):
    # note = json.loads(note)
    item = Note.filter_by(owner_id=user_id,id=note["id"]).first()
    # db.add(db_item)
    item.delete()
    return item
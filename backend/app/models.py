# from sqlalchemy import Boolean, Column, ForeignKey, Integer, String
# from sqlalchemy.orm import relationship
# import hashlib

# from database import Base

from tortoise.models import Model
from tortoise import fields

class Users(Model):

    id = fields.IntField(pk=True)
    email = fields.CharField(max_length=256)
    hashed_password = fields.CharField(max_length=128)
    salt= fields.CharField(max_length=128)
    is_active = fields.BooleanField(default=True)
    


class Item(Model):

    id = fields.IntField(pk=True)
    title = fields.CharField(max_length=128, index=True)
    description = fields.CharField(max_length=128, index=True)
    owner_id = fields.ForeignKeyField("models.Users")


class Note(Model):

    id = fields.IntField(pk=True)
    title = fields.CharField(max_length=128, index=True)
    description = fields.CharField(max_length=128, index=True)
    owner_id = fields.ForeignKeyField("models.Users")

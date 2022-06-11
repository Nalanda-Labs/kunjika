from dataclasses import field
from enum import unique
from tortoise.models import Model
from tortoise import fields


class Users(Model):

    id = fields.IntField(pk=True)
    email = fields.CharField(max_length=256, unique=True)
    username = fields.CharField(max_length=64, unique=True)
    hashed_password = fields.CharField(max_length=128)
    salt = fields.CharField(max_length=128)
    email_verified = fields.BooleanField(default=False)
    is_active = fields.BooleanField(default=True)
    created_at = fields.DatetimeField(auto_now_add=True)
    modified_at = fields.DatetimeField(auto_now=True)

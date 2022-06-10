from fastapi import APIRouter

router = APIRouter()

# client = Minio("s3.amazonaws.com", "ACCESS-KEY", "SECRET-KEY")


@router.get("/api/files")
async def get_events():
    # response = await get_bookmarks()

    # print(response)

    return {"message":"Hello World"}
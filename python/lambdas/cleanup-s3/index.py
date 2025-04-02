import json
import time

import boto3
s3r = boto3.resource('s3')
s3c = boto3.client('s3')

def clean_s3_website(s3_bucket_name: str) -> None:
    """
    Clean up old deployments in an S3 bucket by marking objects for deletion

    Args:
        s3_bucket_name: Name of the S3 bucket to clean
    """
    print(f"Cleaning S3 Bucket: {s3_bucket_name}")
    website_bucket = s3r.Bucket(s3_bucket_name)

    # List all objects and sort by creation date (newest first)
    all_objects = list(website_bucket.objects.all())
    all_objects.sort(key=lambda o: o.last_modified, reverse=True)

    # Find most recent codebuild-buildarn
    codebuild_arn = None
    for obj in all_objects:
        s3_object = obj.Object()
        metadata = s3_object.metadata
        # If we don't have a codebuild_arn yet, take the first one
        if codebuild_arn is None and 'codebuild-buildarn' in metadata:
            codebuild_arn = metadata['codebuild-buildarn']
            print(f"Most recent deployment: codebuild_arn={codebuild_arn}")

        # Mark objects for deletion if they don't match the latest deployment
        if 'codebuild-buildarn' not in metadata or codebuild_arn != metadata['codebuild-buildarn']:
            print(f"Marking for deletion: {s3_object.key}")
            s3c.put_object_tagging(
                Bucket=s3_bucket_name,
                Key=s3_object.key,
                Tagging={'TagSet': [{'Key': 'need-to-delete', 'Value': 'true'}]}
            )

def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    # Retrieve the bucket name
    bucket_name = event['Records'][0]['s3']['bucket']['name']

    print("Sleeping 10 seconds to ensure invoking deployment is finished")
    time.sleep(10)

    clean_s3_website(bucket_name)

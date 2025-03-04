import json
import time

import boto3
s3r = boto3.resource('s3')
s3c = boto3.client('s3')

def clean_s3_website(s3_bucket_name):
    print(f"Cleaning S3 Bucket: {s3_bucket_name}")
    website_bucket = s3r.Bucket(s3_bucket_name)

    # List all the objects and sort them by creation date (first = more recent)
    all_objects = list(website_bucket.objects.all())
    all_objects.sort(key=lambda o:o.last_modified, reverse=True)

    # Now, the bucket will contains files that all have the same codebuild-buildarn if they are part of the same deployment
    # So, with the objects ordered by creation date, the first codebuild-buildarn we encounter is the most recent one
    codebuild_arn = None
    for o in all_objects:
        s3_object = o.Object()
        # If we don't have a codebuild_arn yet, take the first one
        if codebuild_arn is None and 'codebuild-buildarn' in s3_object.metadata:
            codebuild_arn = s3_object.metadata['codebuild-buildarn']
            print(f"most recent deployment is: codebuild_arn={codebuild_arn}")
        
        # If the object does not have the metadata or another value, delete it
        if 'codebuild-buildarn' not in s3_object.metadata or codebuild_arn != s3_object.metadata['codebuild-buildarn']:
            print(f"mark for deletion: {s3_object.key}")
            s3c.put_object_tagging(
                Bucket=s3_bucket_name,
                Key=s3_object.key,
                Tagging={'TagSet': [{'Key': 'need-to-delete', 'Value': 'true'}]}
            )

def lambda_handler(event, context):
    print(json.dumps(event, default=str))
    # Retrieve the bucket name
    bucket_name = event['Records'][0]['s3']['bucket']['name']
    print(f"Sleeping 10 seconds to maximise chances the invoking deployment is finished")
    time.sleep(10)
    clean_s3_website(bucket_name)

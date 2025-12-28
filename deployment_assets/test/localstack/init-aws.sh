#!/bin/bash
awslocal s3 mb s3://test-bucket
awslocal iam create-role --role-name PangolinRole --assume-role-policy-document file://trust-policy.json

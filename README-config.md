## 📦 S3 - Garage

To start Garage (S3 compatible service), you need to set up the following environment variables :
```
AWS_ACCESS_KEY_ID=your_access_key_here
AWS_SECRET_ACCESS_KEY=your_secret_key_here
AWS_DEFAULT_REGION=garage
```

To use it locally, you can install the [AWS CLI](https://github.com/aws/aws-cli). To work properly, the `aws` command need you to export the previous environment variables in your current shell:
```
export AWS_ACCESS_KEY_ID=your_access_key_here
export AWS_SECRET_ACCESS_KEY=your_secret_key_here
export AWS_DEFAULT_REGION=garage
export AWS_ENDPOINT_URL=http://localhost:3900
```

You can now use all `aws s3` commands available ([here is a cheat sheet](https://gist.github.com/cereblanco/5d1dc6687d426d644c02141d0de90ef0)).

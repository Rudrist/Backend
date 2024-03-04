# Current status

## Completed the implementation of the account API, except for the "forget password" feature.

The Get method for function "forget password" has been finsihed.
And the reset password email can be sent correctly.
Currently facing challenges in redirecting users to the correct page via the link provided in the reset password email. 
Further discussion are needed to resolve this.

## Fix some connection error
The database can be implemented by docker or by postgreSQL.
However due to some network port error, database implemented by docker can not be correctly connected (you will find 503 error in your page)
As the result, it is recommended for user to install PostgreSQL, and use "diesel migration run" to create database.




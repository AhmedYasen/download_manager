# Download Manager

## Some Features
- Run concurrently
- Control number of threads
- Set Path and the manager will create it recursively if not exist
- Set custom file name (other than the download file name)
- The manager append time stamp if you download two file with the same name
- Set custom download path for each file if you want
- Run through cmd or by restful apis

## Usage

 1. Open the cmd at the exe file and run one of the commands below
 2. Open postman and click import from the collection tab at the left or check the requests below

 
## Commands 
** ( values in <> is mandatory | values in [ ] is optional) **

			- manager help
			- manager -h
			- manager <subcommand> -h
			- manager start -a <active_downloads> -p <download_path>
			- manager add -u <url> -p [custom_download_path] -f [custom_filename]
			- manager list active
			- manager list all
			- manager list done
			- manager info -f <filename>

## RESTApi
		- all requests must be sent to http://127.0.0.1/command
		- all requests are POST requests
		- all requests written in JSON format
		- Requests (I will provide postman file see postman folder):
			- Add:
			{
				"subcommands": {
					"Add": {
					"url": "http://212.183.159.230/5MB.zip",
					"custom_name": "file_one",
					"custom_download_path": "./down_path"
				}
			}
			- List All
			{
				"subcommands": {
					"List" : {
						"subcommands": "All"
					}
				}
			}
			- List Active
			{
				"subcommands": {
					"List" : {
						"subcommands": "Active"
					}
				}
			}
			- List Done
			{
				"subcommands": {
					"List" : {
						"subcommands": "Done"
					}
				}
			}
			- Info of a file
			{
				"subcommands": {
					"Info" : {
						"filename": "<filename.ext>"
					}
				}
			}




## System Arc
see the doc folder
# build the project
sh package.sh --local
# cd into flappy_goud and restore packages from the local feed
cd examples/flappy_goud
dotnet restore --source $HOME/nuget-local
dotnet build
dotnet run

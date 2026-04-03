PROFILE_DURATION ?= 10
NPC_COUNT ?= 200
ANIMAL_COUNT ?= 80
SHADOW_SIZE ?= 2048

# Allow net8 apps to run on newer .NET runtimes
export DOTNET_ROLL_FORWARD := Major

.PHONY: profile profile-stress run

profile:
	dotnet run --project . -- --profile --duration $(PROFILE_DURATION) \
		--npcs $(NPC_COUNT) --animals $(ANIMAL_COUNT) \
		--shadows --shadow-size $(SHADOW_SIZE)

profile-stress:
	dotnet run --project . -- --profile --duration $(PROFILE_DURATION) \
		--npcs 400 --animals 100 \
		--shadows --shadow-size $(SHADOW_SIZE)

run:
	dotnet run --project .

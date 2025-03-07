<script setup>
import { RouterView } from 'vue-router';
import ThemeSwitch from '@/components/ThemeSwitch.vue';
import LayoutLinks from '@/components/LayoutLinks.vue';
import { computed, inject, onMounted, onUnmounted, provide, reactive, ref, watch } from 'vue';
import { alert_appsync_error, alert_error } from '@/modules/utils';

const client = inject('appsync_client');

const game_status = ref(null);
provide('game_status', game_status);
const players = reactive(new Map());
provide('players', players);
const teams = reactive(new Map());
provide('teams', teams);

function update_game_status(status) {
  game_status.value = status;
  if (status == 'RESET') {
    reset_game();
  }
}
function reset_game() {
  players.forEach((p) => {
    p.clicks = 0;
    p.avg_latency = NaN;
    p.avg_latency_clicks = 0;
  });
  teams.forEach((t) => {
    t.total_clicks = 0;
    t.avg_latency = NaN;
    t.avg_latency_clicks = 0;
  });
}

const registered_player_id = inject('registered_player_id');
watch(players, () => {
  control_player_id();
});
function control_player_id() {
  if (registered_player_id.value && !players.has(registered_player_id.value)) {
    // If we have an ID but it is not in the game state,
    // better forget it...
    registered_player_id.value = null;
  }
}
const current_player = computed(() => {
  return players.get(registered_player_id.value);
});
provide('current_player', current_player);

function uncount_player(team, player) {
  if (player.clicks > 0) {
    // Remove the share of click of the previous version
    team.total_clicks -= player.clicks;
    // Compute the avg_latency_clicks without the player
    const old_avg_latency_clicks = team.avg_latency_clicks - player.avg_latency_clicks;
    // Compute the total_added_latency without the player
    const old_total_added_latency =
      team.avg_latency * team.avg_latency_clicks - player.avg_latency * player.avg_latency_clicks;

    // Update the team fields
    team.avg_latency = old_avg_latency_clicks
      ? old_total_added_latency / old_avg_latency_clicks
      : NaN;
    team.avg_latency_clicks = old_avg_latency_clicks;
  }
}
function count_player(team, player) {
  if (player.clicks > 0) {
    // Add players clicks to the total
    team.total_clicks += player.clicks;
    // Compute the player's avg_latency_clicks
    const avg_latency_clicks = team.avg_latency_clicks + player.avg_latency_clicks;
    // Compute the total_added_latency with the player
    const total_added_latency =
      (team.avg_latency_clicks ? team.avg_latency * team.avg_latency_clicks : 0.0) +
      player.avg_latency * player.avg_latency_clicks;

    // Update the team fields
    team.avg_latency = total_added_latency / avg_latency_clicks;
    team.avg_latency_clicks = avg_latency_clicks;
  }
}
function remove_player(player_id) {
  const player = players.get(player_id);
  if (players.delete(player_id)) {
    const team = teams.get(player.team);
    uncount_player(team, player);
    team.players_count -= 1;
  }
}
function update_player(player) {
  const team_name = player.team;

  // Retrieve the team object
  let team = teams.get(team_name);
  // If this is the first time we see the team_name,
  // Init the team object
  if (team == null) {
    teams.set(team_name, {
      team_name,
      players_count: 0,
      total_clicks: 0,
      avg_latency: NaN,
      avg_latency_clicks: 0,
    });
    // We set and then re-get to retrieve the Proxied object (we are in a reactive context)
    team = teams.get(team_name);
  }

  // If the player was already present, uncount them first
  const existing_player = players.get(player.id);
  if (existing_player != null) {
    uncount_player(team, existing_player);
  } else {
    // If the player was not present, it is the first time we saw them
    // So add them to the team list
    team.players_count += 1;
  }

  // Then count them in
  count_player(team, player);

  // And finally update the players Map
  players.set(player.id, player);
}

async function load_game_state() {
  try {
    const gs = (
      await client.graphql({
        query: `
        query GameState {
          gameState {
            status
            players {
              id
              name
              team
              clicks
              avg_latency
              avg_latency_clicks
            }
          }
        }
      `,
      })
    ).data.gameState;
    console.log(gs);
    update_game_status(gs.status);
    gs.players.forEach((p) => {
      update_player(p);
    });
    control_player_id();
  } catch (e) {
    alert_appsync_error(e, 'Could not retrieve the Game state 😭');
  }

  try {
    subscribe_updates();
  } catch (e) {
    console.error(e);
    alert_error('Could subscribe to live update 😭');
  }
}

const subscriptions = new Array();
function subscribe_updates() {
  console.log('subscribe_updates BEGIN');
  unsubscribe_updates();
  console.log('Subscribing updatedPlayer');
  const updated_player = client
    .graphql({
      query: `
        subscription UpdatedPlayer {
          updatedPlayer {
            id
            name
            team
            clicks
            avg_latency
            avg_latency_clicks
          }
        }
      `,
    })
    .subscribe({
      next: ({ data }) => {
        console.log(data);
        if (data.updatedPlayer) {
          const player = data.updatedPlayer;
          update_player(player);
        }
      },
      error: (error) => console.error(error),
    });
  console.log('Subscribing removedPlayer');
  const removed_player = client
    .graphql({
      query: `
        subscription RemovedPlayer {
          removedPlayer {
            id
            name
            team
            clicks
            avg_latency
            avg_latency_clicks
          }
        }
      `,
    })
    .subscribe({
      next: ({ data }) => {
        console.log(data);
        if (data.removedPlayer) {
          const player = data.removedPlayer;
          remove_player(player.id);
        }
      },
      error: (error) => console.error(error),
    });
  console.log('Subscribing updatedGameStatus');
  const updated_game_status = client
    .graphql({
      query: `
        subscription UpdatedGameStatus {
          updatedGameStatus
        }
      `,
    })
    .subscribe({
      next: ({ data }) => {
        console.log(data);
        if (data.updatedGameStatus) {
          const status = data.updatedGameStatus;
          update_game_status(status);
        }
      },
      error: (error) => console.error(error),
    });

  subscriptions.push(...[updated_player, removed_player, updated_game_status]);
  console.log('subscribe_updates END');
}

function unsubscribe_updates() {
  console.log('unsubscribe_updates BEGIN');
  subscriptions.forEach((sub) => {
    console.log('Unsubscribing...');
    sub.unsubscribe();
    console.log(sub);
  });
  subscriptions.splice(0, subscriptions.length);
  console.log('unsubscribe_updates END');
}

onMounted(async () => {
  console.log('onMounted BEGIN');
  await load_game_state();
  console.log('onMounted END');
});
onUnmounted(() => {
  console.log('onUnmounted BEGIN');
  unsubscribe_updates();
  console.log('onUnmounted END');
});
</script>

<template>
  <div class="flex flex-col min-h-screen">
    <header class="h-16">
      <!-- Navbar -->
      <nav
        class="fixed top-0 left-0 z-10 bg-base-100/90 navbar py-0 shadow-sm shadow-base-content/50 w-full h-16"
      >
        <div class="navbar-start">
          <img src="@/assets/logo.webp" class="h-14" />
          <div class="text-2xl font-black ml-2">
            <div>Benchmark</div>
            <div>Game</div>
          </div>
        </div>
        <div class="navbar-center hidden md:inline-flex gap-6">
          <layout-links></layout-links>
        </div>
        <div class="navbar-end">
          <theme-switch></theme-switch>
          <div class="basis-4"></div>
        </div>
      </nav>
    </header>
    <!-- Page content here -->
    <router-view />
    <!-- Footer -->
    <div class="grow"></div>
    <footer class="footer justify-center md:justify-end pb-20 pt-4 md:py-4 md:p-4">
      <aside class="text-xs">
        <a
          href="https://github.com/JeremieRodon/demo-rust-lambda-appsync.git"
          class="p-1"
          target="_blank"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            class="h-6 inline fill-current"
          >
            <path
              d="M12,2A10,10 0 0,0 2,12C2,16.42 4.87,20.17 8.84,21.5C9.34,21.58 9.5,21.27 9.5,21C9.5,20.77 9.5,20.14 9.5,19.31C6.73,19.91 6.14,17.97 6.14,17.97C5.68,16.81 5.03,16.5 5.03,16.5C4.12,15.88 5.1,15.9 5.1,15.9C6.1,15.97 6.63,16.93 6.63,16.93C7.5,18.45 8.97,18 9.54,17.76C9.63,17.11 9.89,16.67 10.17,16.42C7.95,16.17 5.62,15.31 5.62,11.5C5.62,10.39 6,9.5 6.65,8.79C6.55,8.54 6.2,7.5 6.75,6.15C6.75,6.15 7.59,5.88 9.5,7.17C10.29,6.95 11.15,6.84 12,6.84C12.85,6.84 13.71,6.95 14.5,7.17C16.41,5.88 17.25,6.15 17.25,6.15C17.8,7.5 17.45,8.54 17.35,8.79C18,9.5 18.38,10.39 18.38,11.5C18.38,15.32 16.04,16.16 13.81,16.41C14.17,16.72 14.5,17.33 14.5,18.26C14.5,19.6 14.5,20.68 14.5,21C14.5,21.27 14.66,21.59 15.17,21.5C19.14,20.16 22,16.42 22,12A10,10 0 0,0 12,2Z"
            />
          </svg>
          Deploy your own!
        </a>
      </aside>
    </footer>
    <div class="dock bg-base-100 flex-none md:hidden h-16 z-20">
      <layout-links></layout-links>
    </div>
  </div>
</template>

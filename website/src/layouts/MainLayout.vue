<script setup>
import { RouterView } from 'vue-router';
import ThemeSwitch from '@/components/ThemeSwitch.vue';
import LayoutLinks from '@/components/LayoutLinks.vue';
import { computed, inject, onMounted, onUnmounted, provide, ref, watch, watchEffect } from 'vue';
import { alert_appsync_error, alert_error } from '@/modules/utils';

const client = inject('appsync_client');

const commit_buffer_interval = { v: null };
const players_change_buffer = new Map();
const players_remove_buffer = new Set();
function commit_players_buffer() {
  console.log('commit_players_buffer');
  if (
    players_change_buffer.size == 0 &&
    players_remove_buffer.size == 0 &&
    commit_buffer_interval.v != null
  ) {
    stop_updates_processing();
  } else {
    // Extract the player map from the ref value
    const players_map = players.value;

    // Apply updates if any
    players_change_buffer.forEach((player) => {
      players_map.set(player.id, player);
    });
    players_change_buffer.clear();

    // Apply deletes if any
    players_remove_buffer.forEach((player_id) => {
      players_map.delete(player_id);
    });
    players_remove_buffer.clear();

    // Change the ref value with the new player map (will trigger Vue updates)
    players.value = players_map;
  }
}
function start_updates_processing() {
  console.log('start_updates_processing');
  if (commit_buffer_interval.v == null) {
    console.log('actually_start_updates_processing');
    commit_players_buffer();
    commit_buffer_interval.v = setInterval(commit_players_buffer, 500);
  }
}
function stop_updates_processing() {
  console.log('stop_updates_processing');
  if (commit_buffer_interval.v != null) {
    console.log('actually_stop_updates_processing');
    clearInterval(commit_buffer_interval.v);
    commit_buffer_interval.v = null;
  }
}

const game_status = ref(null);
provide('game_status', game_status);
function update_game_status(status) {
  console.log('update_game_status');
  game_status.value = status;
  if (status == 'RESET') {
    reset_game();
  }
}

const players = ref(new Map());
const ranked_players = ref(null);
provide('ranked_players', ranked_players);
watchEffect(() => {
  console.log('COMPUTED ranked_players');
  const ranked_players_tmp = [...players.value.values()];
  ranked_players_tmp.sort((p1, p2) => {
    const s = p2.clicks - p1.clicks;
    if (s == 0) {
      return p1.name.localeCompare(p2.name);
    } else {
      return s;
    }
  });
  ranked_players_tmp.forEach((p, i, rp) => {
    if (i > 0) {
      const previous_p = rp[i - 1];
      if (p.clicks == previous_p.clicks) {
        p.rank = previous_p.rank;
        return;
      }
    }
    if (p.clicks) {
      p.rank = i + 1;
    } else {
      p.rank = null;
    }
  });
  ranked_players.value = ranked_players_tmp;
});

const teams = computed(() => {
  console.log('COMPUTED teams');
  const teams = new Map();
  for (const team_name of ['RUST', 'PYTHON', 'JS', 'VTL']) {
    teams.set(team_name, {
      team_name,
      players_count: 0,
      total_clicks: 0,
      total_latency: 0.0,
      avg_latency_clicks: 0,
    });
  }
  for (const player of players.value.values()) {
    console.log(player);
    const team_name = player.team;
    const total_clicks = player.clicks ? player.clicks : 0;
    const avg_latency_clicks = player.avg_latency_clicks ? player.avg_latency_clicks : 0;
    const total_latency = player.avg_latency ? player.avg_latency * avg_latency_clicks : 0.0;

    const team_obj = teams.get(team_name);
    team_obj.players_count += 1;
    team_obj.total_clicks += total_clicks;
    team_obj.total_latency += total_latency;
    team_obj.avg_latency_clicks += avg_latency_clicks;
  }
  for (const team_obj of teams.values()) {
    console.log(team_obj);
    const avg_latency = team_obj.total_latency / team_obj.avg_latency_clicks;
    team_obj.avg_latency = avg_latency;
    delete team_obj.total_latency;
  }
  return teams;
});
provide('teams', teams);

const ranked_teams = ref(null);
provide('ranked_teams', ranked_teams);
watchEffect(() => {
  console.log('COMPUTED ranked_teams');
  const ranked_teams_tmp = [...teams.value.values()];
  ranked_teams_tmp.sort((t1, t2) => {
    let s1 = 0;
    if (isNaN(t1.avg_latency) && isNaN(t2.avg_latency)) {
      s1 = 0;
    } else if (isNaN(t1.avg_latency)) {
      s1 = 1;
    } else if (isNaN(t2.avg_latency)) {
      s1 = -1;
    } else {
      s1 = t1.avg_latency - t2.avg_latency;
    }
    if (s1 == 0) {
      return t1.team_name.localeCompare(t2.team_name);
    } else {
      return s1;
    }
  });
  ranked_teams_tmp.forEach((t, i, rt) => {
    if (i > 0) {
      const previous_t = rt[i - 1];
      if (
        t.avg_latency == previous_t.avg_latency ||
        (isNaN(t.avg_latency) && isNaN(previous_t.avg_latency))
      ) {
        t.rank = previous_t.rank;
        return;
      }
    }
    t.rank = i + 1;
  });
  ranked_teams.value = ranked_teams_tmp;
});

function reset_game() {
  const players_map = players.value;
  players_map.forEach((p) => {
    p.clicks = 0;
    p.avg_latency = NaN;
    p.avg_latency_clicks = 0;
  });
  players.value = new Map(players_map);
}

const registered_player_obj = inject('registered_player_obj');
watch(players, () => {
  control_player_id();
});
function control_player_id() {
  if (registered_player_obj.value && !players.value.has(registered_player_obj.value.player_id)) {
    // If we have an ID but it is not in the game state,
    // better forget it...
    registered_player_obj.value = null;
  }
}

const current_player = computed(() => {
  return players.value.get(registered_player_obj.value?.player_id);
});
provide('current_player', current_player);

async function load_game_state() {
  try {
    const gs = (
      await client.graphql({
        query: `
        query GameState {
          status: gameStatus
          players: players {
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
    ).data;
    console.log(gs);
    update_game_status(gs.status);
    const players_map = new Map();
    gs.players.forEach((p) => {
      players_map.set(p.id, p);
    });
    players.value = players_map;
  } catch (e) {
    alert_appsync_error(e, 'Could not retrieve the Game state 😭');
  }
}

const subscriptions = new Array();
function subscribe_updates() {
  console.log('subscribe_updates BEGIN');
  unsubscribe_updates();
  try {
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
          const player = data.updatedPlayer;
          players_change_buffer.set(player.id, player);
          start_updates_processing();
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
            players_remove_buffer.add(player.id);
            start_updates_processing();
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
  } catch (e) {
    console.error(e);
    alert_error('Could subscribe to live update 😭');
  }

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
  console.log('MainLayout onMounted BEGIN');
  subscribe_updates();
  await load_game_state();
  start_updates_processing();
  console.log('MainLayout onMounted END');
});
onUnmounted(() => {
  console.log('MainLayout onUnmounted BEGIN');
  unsubscribe_updates();
  stop_updates_processing();
  console.log('MainLayout onUnmounted END');
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

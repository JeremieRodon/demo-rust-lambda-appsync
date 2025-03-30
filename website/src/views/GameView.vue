<script setup>
import CChangeName from '@/components/CChangeName.vue';
import CRegistration from '@/components/CRegistration.vue';
import TeamIcon from '@/components/TeamIcon.vue';
import DisplayClicks from '@/components/DisplayClicks.vue';
import DisplayLatency from '@/components/DisplayLatency.vue';
import DisplayRank from '@/components/DisplayRank.vue';
import { alert_appsync_error, team_to_displayname } from '@/modules/utils';
import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue';
import DisplayPlayerCount from '@/components/DisplayPlayerCount.vue';

const registered_player_obj = inject('registered_player_obj');
const current_player = inject('current_player');
const game_status = inject('game_status');
const client = inject('appsync_client');

const teams = inject('teams');

const signed_user_is_admin = inject('signed_user_is_admin');
const register_modal_force_close = ref(signed_user_is_admin.value);
const change_name_modal_open = ref(false);

const local_clicks_counter = ref(0);

const current_player_name = computed(() => {
  if (current_player.value) {
    return current_player.value.name;
  } else {
    return 'PlaceHolder';
  }
});
const current_player_team_id = computed(() => {
  if (current_player.value) {
    return current_player.value.team;
  } else {
    return null;
  }
});
const current_player_rank = computed(() => {
  if (current_player.value) {
    return current_player.value.rank;
  } else {
    return null;
  }
});
const current_player_clicks = computed(() => {
  if (game_status.value == 'STARTED') {
    return local_clicks_counter.value;
  } else {
    if (current_player.value) {
      return current_player.value.clicks;
    } else {
      return 0;
    }
  }
});
const current_player_avg_latency = computed(() => {
  if (current_player.value) {
    return Math.round(current_player.value.avg_latency * 100) / 100;
  } else {
    return NaN;
  }
});

const current_player_team_name = computed(() => {
  if (current_player.value) {
    return team_to_displayname(current_player.value.team);
  } else {
    return '';
  }
});
const current_player_team = computed(() => {
  if (current_player.value) {
    const team = current_player.value.team;
    return teams.value.get(team);
  }
  return null;
});

const current_player_team_players_count = computed(() => {
  if (current_player_team.value) {
    return current_player_team.value.players_count;
  } else {
    return 0;
  }
});
const current_player_team_clicks = computed(() => {
  if (current_player_team.value) {
    return current_player_team.value.total_clicks;
  } else {
    return 0;
  }
});
const current_player_team_avg_latency = computed(() => {
  if (current_player_team.value) {
    return Math.round(current_player_team.value.avg_latency * 100) / 100;
  } else {
    return NaN;
  }
});

const click_mutation = computed(() => {
  switch (current_player.value.team) {
    case 'RUST': {
      return 'clickRust';
    }
    case 'PYTHON': {
      return 'clickPython';
    }
    case 'JS': {
      return 'clickJs';
    }
    case 'VTL': {
      return 'clickVtl';
    }
  }
  return null;
});
async function call_click() {
  local_clicks_counter.value += 1;
  const { player_id, secret } = registered_player_obj.value;
  const variables = {
    player_id,
    secret,
  };
  console.log(variables);

  const start = Date.now();
  try {
    await client.graphql({
      query: `
        mutation Click($player_id: ID!, $secret: String!) {
          ${click_mutation.value}(player_id: $player_id, secret: $secret) {
              id
              name
              team
              clicks
              avg_latency
              avg_latency_clicks
            }
          }
      `,
      variables,
    });
  } catch (e) {
    alert_appsync_error(e, 'Could not click 😭');
  } finally {
    const duration = Date.now() - start;
    latency_report_buffer.push(duration);
  }
}

const latency_report_buffer = new Array();
const report_timer = ref(null);
function begin_reporting() {
  if (report_timer.value == null) {
    report_timer.value = setInterval(async () => await report_latency(), 1000);
  }
}
function stop_reporting() {
  if (report_timer.value != null) {
    clearInterval(report_timer.value);
    report_timer.value = null;
  }
}
watch([game_status, current_player], () => {
  if (current_player.value && game_status.value == 'STARTED') {
    begin_reporting();
  } else {
    stop_reporting();
  }
});

const report_latency_mutation = computed(() => {
  switch (current_player.value.team) {
    case 'RUST': {
      return 'reportLatencyRust';
    }
    case 'PYTHON': {
      return 'reportLatencyPython';
    }
    case 'JS': {
      return 'reportLatencyJs';
    }
    case 'VTL': {
      return 'reportLatencyVtl';
    }
  }
  return null;
});
async function report_latency() {
  console.log('report_latency');
  const { player_id, secret } = registered_player_obj.value;
  const latencies = latency_report_buffer.splice(0, latency_report_buffer.length);
  if (latencies.length == 0) {
    return;
  }
  const clicks = latencies.length;
  const avg_latency = latencies.reduce((acc, c) => acc + c, 0) / latencies.length;
  const report = {
    clicks,
    avg_latency,
  };
  const variables = {
    player_id,
    report,
    secret,
  };
  console.log(variables);

  try {
    await client.graphql({
      query: `
        mutation ReportLatency($player_id: ID!, $report: LatencyReport!, $secret: String!) {
          ${report_latency_mutation.value}(player_id: $player_id, report: $report, secret: $secret) {
              id
              name
              team
              clicks
              avg_latency
              avg_latency_clicks
            }
          }
      `,
      variables,
    });
  } catch (e) {
    alert_appsync_error(e, 'Could not report 😭');
  }
}
onMounted(() => {
  console.log('GameView onMounted BEGIN');
  if (current_player.value && game_status.value == 'STARTED') {
    begin_reporting();
  }
  console.log('GameView onMounted END');
});
onUnmounted(() => {
  console.log('GameView onUnmounted BEGIN');
  stop_reporting();
  console.log('GameView onUnmounted END');
});
</script>

<template>
  <main>
    <div class="px-2 md:px-4 mx-auto max-w-screen-lg">
      <div class="flex flex-col m-2 sm:m-4">
        <div class="flex flex-row justify-center items-center gap-2">
          <team-icon :name="current_player_team_id" class="h-16"></team-icon>
          <div class="flex flex-col items-start">
            <div class="text-xl font-black text-wrap [word-break:break-word]">
              {{ current_player_name }}
            </div>
            <div class="text-sm font-light text-nowrap overflow-visible">
              {{ current_player_team_name }}
            </div>
          </div>
          <button class="btn btn-circle btn-ghost" @click="change_name_modal_open = true">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-8 fill-primary" viewBox="0 0 24 24">
              <path
                d="M19.71,8.04L17.37,10.37L13.62,6.62L15.96,4.29C16.35,3.9 17,3.9 17.37,4.29L19.71,6.63C20.1,7 20.1,7.65 19.71,8.04M3,17.25L13.06,7.18L16.81,10.93L6.75,21H3V17.25M16.62,5.04L15.08,6.58L17.42,8.92L18.96,7.38L16.62,5.04M15.36,11L13,8.64L4,17.66V20H6.34L15.36,11Z"
              />
            </svg>
          </button>
        </div>
        <button
          v-if="signed_user_is_admin && !current_player"
          class="btn btn-primary w-40 mx-auto"
          @click="register_modal_force_close = !register_modal_force_close"
        >
          Admin Register
        </button>

        <div class="grid grid-cols-5 w-fit mx-auto">
          <div class="divider text-base-content/60 font-bold text-sm mb-0 col-span-5">
            Player stats
          </div>
          <div class="col-span-5 grid grid-cols-subgrid justify-items-center gap-4">
            <display-rank :rank="current_player_rank"></display-rank>
            <display-clicks :clicks="current_player_clicks" class="col-span-2"></display-clicks>
            <display-latency
              :avg_latency="current_player_avg_latency"
              class="col-span-2"
            ></display-latency>
          </div>
          <div class="divider text-base-content/60 font-bold text-sm mb-0 col-span-5">
            Team stats
          </div>
          <div class="col-span-5 grid grid-cols-subgrid justify-items-center gap-4">
            <display-player-count
              :player_count="current_player_team_players_count"
            ></display-player-count>
            <display-clicks
              :clicks="current_player_team_clicks"
              class="col-span-2"
            ></display-clicks>
            <display-latency
              :avg_latency="current_player_team_avg_latency"
              class="col-span-2"
            ></display-latency>
          </div>
        </div>
      </div>

      <div class="max-w-lg mx-auto">
        <button
          class="btn btn-block uppercase btn-secondary font-black text-5xl py-16 sm:py-20"
          :disabled="game_status != 'STARTED' || !current_player"
          @click="call_click"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            class="h-12 w-12 fill-current"
          >
            <path
              d="M2 19.63L13.43 8.2L12.72 7.5L14.14 6.07L12 3.89C13.2 2.7 15.09 2.7 16.27 3.89L19.87 7.5L18.45 8.91H21.29L22 9.62L18.45 13.21L17.74 12.5V9.62L16.27 11.04L15.56 10.33L4.13 21.76L2 19.63Z"
            />
          </svg>
          Smash!
        </button>
      </div>
    </div>
    <c-registration :force_close="register_modal_force_close"></c-registration>
    <c-change-name v-model="change_name_modal_open"></c-change-name>
  </main>
</template>

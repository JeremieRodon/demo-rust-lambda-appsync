<script setup>
import CChangeName from '@/components/CChangeName.vue';
import CRegistration from '@/components/CRegistration.vue';
import TeamIcon from '@/components/TeamIcon.vue';
import TeamScore from '@/components/TeamScore.vue';
import { alert_appsync_error, team_to_displayname } from '@/modules/utils';
import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue';

const current_player = inject('current_player');
const teams = inject('teams');
const game_status = inject('game_status');
const client = inject('appsync_client');

const change_name_modal_open = ref(false);

watch(game_status, () => {
  if (game_status.value == 'STARTED') {
    begin_reporting();
  } else {
    stop_reporting();
  }
});

const current_player_id = computed(() => {
  if (current_player.value) {
    return current_player.value.id;
  } else {
    return null;
  }
});
const current_player_name = computed(() => {
  if (current_player.value) {
    return current_player.value.name;
  } else {
    return 'PlaceHolder';
  }
});
const current_player_team = computed(() => {
  if (current_player.value) {
    return current_player.value.team;
  } else {
    return null;
  }
});
const current_player_team_name = computed(() => {
  return team_to_displayname(current_player_team.value);
});
const sorted_teams = computed(() => {
  const sorted_teams = [...teams.values()];
  sorted_teams.sort((t1, t2) => t1.avg_latency - t2.avg_latency);
  return sorted_teams;
});

const latency_report_buffer = new Array();
async function call_click() {
  const player_id = current_player_id.value;
  const variables = {
    player_id,
  };
  console.log(variables);
  let click_mutation;
  switch (current_player_team.value) {
    case 'RUST': {
      click_mutation = 'clickRust';
      break;
    }
    case 'JS': {
      click_mutation = 'clickJs';
      break;
    }
    case 'VTL': {
      click_mutation = 'clickVtl';
      break;
    }
  }
  const start = Date.now();
  try {
    await client.graphql({
      query: `
        mutation click($player_id: ID!) {
          ${click_mutation}(player_id: $player_id) {
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
async function report_latency() {
  console.log('report_latency');
  const player_id = current_player_id.value;
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
  };
  console.log(variables);
  let report_latency_mutation;
  switch (current_player_team.value) {
    case 'RUST': {
      report_latency_mutation = 'reportLatencyRust';
      break;
    }
    case 'JS': {
      report_latency_mutation = 'reportLatencyJs';
      break;
    }
    case 'VTL': {
      report_latency_mutation = 'reportLatencyVtl';
      break;
    }
  }
  try {
    await client.graphql({
      query: `
        mutation reportLatency($player_id: ID!, $report: LatencyReport!) {
          ${report_latency_mutation}(player_id: $player_id, report: $report) {
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
  console.log('onMounted BEGIN');
  if (game_status.value == 'STARTED') {
    begin_reporting();
  }
  console.log('onMounted END');
});
onUnmounted(() => {
  console.log('onUnmounted BEGIN');
  stop_reporting();
  console.log('onUnmounted END');
});
</script>

<template>
  <main>
    <div class="px-2 md:px-4 mx-auto max-w-screen-lg">
      <div class="flex flex-row justify-center items-center m-4 gap-2">
        <team-icon :name="current_player_team" class="h-16"></team-icon>
        <div class="flex flex-col items-start">
          <div class="text-xl font-black text-wrap [word-break:break-word]">
            {{ current_player_name }}
          </div>
          <div class="text-sm font-light max-w-20 text-nowrap overflow-visible">
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
      <div class="max-w-4xl mx-auto">
        <template v-for="team in sorted_teams" :key="team.team_name">
          <team-score :team="team"></team-score>
        </template>
      </div>
      <div class="w-fit mx-auto my-8">
        <button
          class="btn uppercase btn-secondary font-black text-4xl py-16 px-8"
          :disabled="game_status != 'STARTED'"
          @click="call_click"
        >
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" class="h-16 fill-current">
            <path
              d="M2 19.63L13.43 8.2L12.72 7.5L14.14 6.07L12 3.89C13.2 2.7 15.09 2.7 16.27 3.89L19.87 7.5L18.45 8.91H21.29L22 9.62L18.45 13.21L17.74 12.5V9.62L16.27 11.04L15.56 10.33L4.13 21.76L2 19.63Z"
            />
          </svg>
          Smash!!
        </button>
      </div>
    </div>
    <c-registration></c-registration>
    <c-change-name v-model="change_name_modal_open"></c-change-name>
  </main>
</template>

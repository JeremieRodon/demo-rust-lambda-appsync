<script setup>
import { computed, inject } from 'vue';
import TeamIcon from './TeamIcon.vue';
import { team_to_displayname } from '@/modules/utils';

const props = defineProps({
  team: Object,
});

const current_player = inject('current_player');
const is_current_player_team = computed(() => {
  if (current_player.value) {
    return current_player.value.team == props.team.team_name;
  } else {
    return false;
  }
});

const team_name = computed(() => {
  return props.team.team_name;
});
const team_display_name = computed(() => {
  return team_to_displayname(team_name.value);
});
const team_avg_latency_rounded = computed(() => {
  if (isNaN(props.team.avg_latency)) {
    return '-';
  } else {
    return Math.round(props.team.avg_latency * 100) / 100;
  }
});
</script>

<template>
  <div
    class="flex flex-row max-w-sm mx-auto sm:max-w-full m-2 p-1 rounded-box"
    :class="is_current_player_team ? 'border-2 border-secondary' : 'border'"
  >
    <div class="flex flex-col sm:flex-row items-center">
      <div class="text-lg font-bold self-center text-center sm:ml-2">#{{ team.rank }}</div>
      <team-icon :name="team.team_name" class="h-14 m-2 mt-0 sm:ml-2 sm:m-4 self-end"></team-icon>
    </div>
    <div
      class="grid grid-cols-2 grid-rows-3 sm:grid-cols-6 sm:grid-rows-1 grow gap-2 sm:gap-0 items-center"
    >
      <div
        class="text-base-content/60 font-bold text-end sm:text-center text-sm sm:text-base justify-self-end text-nowrap sm:text-wrap"
      >
        {{ team_display_name }}
      </div>
      <div
        class="flex flex-row items-center sm:justify-center sm:border-r border-dashed border-base-content/30"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          class="inline h-4 w-4 sm:h-8 sm:w-8 fill-current shrink-0"
        >
          <path
            d="M12,15C7.58,15 4,16.79 4,19V21H20V19C20,16.79 16.42,15 12,15M8,9A4,4 0 0,0 12,13A4,4 0 0,0 16,9M11.5,2C11.2,2 11,2.21 11,2.5V5.5H10V3C10,3 7.75,3.86 7.75,6.75C7.75,6.75 7,6.89 7,8H17C16.95,6.89 16.25,6.75 16.25,6.75C16.25,3.86 14,3 14,3V5.5H13V2.5C13,2.21 12.81,2 12.5,2H11.5Z"
          />
        </svg>
        <div
          class="flex flex-row sm:flex-col overflow-hidden items-center gap-1 sm:gap-0 sm:items-start"
        >
          <div class="text-lg sm:text-2xl font-black">{{ team.players_count }}</div>
          <div class="text-base-content/60 text-[8px] sm:text-xs font-light">Players</div>
        </div>
      </div>
      <div
        class="text-base-content/60 font-bold text-end sm:text-center text-sm sm:text-base w-fit justify-self-end"
      >
        Total Clicks
      </div>
      <div
        class="flex flex-row items-center sm:justify-center sm:border-r border-dashed border-base-content/30"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          class="inline h-4 w-4 sm:h-8 sm:w-8 fill-current shrink-0"
        >
          <path
            d="M10.76,8.69A0.76,0.76 0 0,0 10,9.45V20.9C10,21.32 10.34,21.66 10.76,21.66C10.95,21.66 11.11,21.6 11.24,21.5L13.15,19.95L14.81,23.57C14.94,23.84 15.21,24 15.5,24C15.61,24 15.72,24 15.83,23.92L18.59,22.64C18.97,22.46 19.15,22 18.95,21.63L17.28,18L19.69,17.55C19.85,17.5 20,17.43 20.12,17.29C20.39,16.97 20.35,16.5 20,16.21L11.26,8.86L11.25,8.87C11.12,8.76 10.95,8.69 10.76,8.69M15,10V8H20V10H15M13.83,4.76L16.66,1.93L18.07,3.34L15.24,6.17L13.83,4.76M10,0H12V5H10V0M3.93,14.66L6.76,11.83L8.17,13.24L5.34,16.07L3.93,14.66M3.93,3.34L5.34,1.93L8.17,4.76L6.76,6.17L3.93,3.34M7,10H2V8H7V10"
          />
        </svg>
        <div
          class="flex flex-row sm:flex-col overflow-hidden items-center gap-1 sm:gap-0 sm:items-start"
        >
          <div class="text-lg sm:text-2xl font-black">{{ team.total_clicks }}</div>
          <div class="text-base-content/60 text-[8px] sm:text-xs font-light">clicks</div>
        </div>
      </div>
      <div
        class="text-base-content/60 font-bold text-end sm:text-center text-sm sm:text-base w-fit justify-self-end"
      >
        Avg. Latency
      </div>
      <div class="flex flex-row items-center sm:justify-center">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          class="inline h-4 w-4 sm:h-8 sm:w-8 stroke-current shrink-0"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 10V3L4 14h7v7l9-11h-7z"
          ></path>
        </svg>
        <div
          class="flex flex-row sm:flex-col overflow-hidden items-center gap-1 sm:gap-0 sm:items-start"
        >
          <div class="text-lg sm:text-2xl font-black">{{ team_avg_latency_rounded }}</div>
          <div class="text-base-content/60 text-[8px] sm:text-xs font-light">milliseconds</div>
        </div>
      </div>
    </div>
  </div>
</template>

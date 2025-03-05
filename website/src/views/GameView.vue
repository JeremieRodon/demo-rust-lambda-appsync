<script setup>
import CRegistration from '@/components/CRegistration.vue';
import TeamIcon from '@/components/TeamIcon.vue';
import TeamScore from '@/components/TeamScore.vue';
import { team_to_displayname } from '@/modules/utils';
import { computed, inject } from 'vue';

const current_player = inject('current_player');
const teams = inject('teams');
const game_status = inject('game_status');
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
</script>

<template>
  <main>
    <div class="px-2 md:px-4 mx-auto max-w-screen-lg">
      <div class="flex flex-row justify-center items-center m-4">
        <div class="flex flex-col items-center min-w-20 w-20">
          <div class="text-sm font-light max-w-20 text-nowrap overflow-visible">
            {{ current_player_team_name }}
          </div>
          <team-icon :name="current_player_team" class="h-16"></team-icon>
        </div>
        <div class="text-xl font-black text-wrap break-all">{{ current_player_name }}</div>
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
  </main>
</template>

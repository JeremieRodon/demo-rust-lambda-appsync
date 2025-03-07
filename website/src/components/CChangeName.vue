<script setup>
import CModal from '@/components/CModal.vue';
import SimpleInput from '@/components/SimpleInput.vue';
import { alert_appsync_error, alert_success } from '@/modules/utils';
import { computed, inject, ref, watch } from 'vue';
const open = defineModel();

const current_player = inject('current_player');
const current_player_id = computed(() => {
  if (current_player.value) {
    return current_player.value.id;
  } else {
    return undefined;
  }
});
const current_player_name = computed(() => {
  if (current_player.value) {
    return current_player.value.name;
  }
  return undefined;
});
console.log(current_player_name.value);
const player_name = ref(current_player_name.value);
watch(current_player_name, () => {
  player_name.value = current_player_name.value;
  console.log(current_player_name.value);
});
const player_name_error = ref(null);

const in_operation = ref(false);

const client = inject('appsync_client');

async function change_name() {
  in_operation.value = true;
  const player_id = current_player_id.value;
  const new_name = player_name.value;
  const variables = {
    player_id,
    new_name,
  };

  console.log(variables);
  try {
    await client.graphql({
      query: `
        mutation UpdatePlayerName($player_id:ID!, $new_name: String!) {
            updatePlayerName(player_id: $player_id, new_name: $new_name) {
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
    open.value = false;
    alert_success(`Player name successfuly changed`);
  } catch (e) {
    alert_appsync_error(e, 'Could not change your name 😭');
  } finally {
    in_operation.value = false;
  }
}
</script>

<template>
  <c-modal v-model="open">
    <div class="card bg-base-100 w-fit mx-auto">
      <div class="card-title text-4xl font-bold justify-center">Change name</div>
      <div class="card-body w-fit">
        <p class="text-base">You can change your player name here if you wish.</p>
        <simple-input
          v-model="player_name"
          v-model:error="player_name_error"
          tabindex="1"
          name="pseudo"
          placeholder="The GOAT 🐐"
          autocomplete="off"
          :rules="[
            (pseudo) => (pseudo && pseudo.length >= 3) || 'Must be at least 3 chars',
            (pseudo) => pseudo.length < 30 || 'Must be max 30 chars',
          ]"
          @keydown.enter="
            if (!in_operation && !player_name_error && current_player_name != player_name) {
              change_name();
            }
          "
        >
          Player Name
        </simple-input>
        <div class="flex flex-row gap-4 justify-center">
          <button
            class="btn btn-primary mt-2"
            :disabled="in_operation || player_name_error || current_player_name == player_name"
            tabindex="2"
            @click="change_name()"
          >
            <span v-show="in_operation" class="loading loading-spinner loading-md"></span>
            Change
          </button>
          <button class="btn btn-primary mt-2" tabindex="2" @click="open = false">Cancel</button>
        </div>
      </div>
    </div>
  </c-modal>
</template>

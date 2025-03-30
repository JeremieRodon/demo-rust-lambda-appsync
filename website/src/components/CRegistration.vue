<script setup>
import CModal from '@/components/CModal.vue';
import SimpleInput from '@/components/SimpleInput.vue';
import { alert_appsync_error, alert_success } from '@/modules/utils';
import { computed, inject, onMounted, ref, watch } from 'vue';
import { v4 as uuidv4 } from 'uuid';

const props = defineProps({
  force_close: Boolean,
});

const registered_player_obj = inject('registered_player_obj');

const pseudo = ref(null);

const need_registration = computed(() => {
  return registered_player_obj.value == null && !props.force_close;
});

watch(need_registration, () => {
  if (need_registration.value) {
    setTimeout(() => pseudo.value.focus(), 100);
  }
});

const player_name = ref(null);
const player_name_error = ref(null);
const in_operation = ref(false);

const client = inject('appsync_client');

async function handle_register() {
  in_operation.value = true;
  const name = player_name.value;
  const secret = uuidv4();
  const variables = {
    name,
    secret,
  };
  console.log(variables);
  try {
    const player_id = (
      await client.graphql({
        query: `
        mutation RegisterNewPlayer($name: String!, $secret: String!) {
            registerNewPlayer(name: $name, secret: $secret) {
              id
              name
              team
            }
          }
      `,
        variables,
      })
    ).data.registerNewPlayer.id;

    registered_player_obj.value = {
      player_id,
      secret,
    };

    alert_success(`Welcome ${name}! 🎉`);
  } catch (e) {
    alert_appsync_error(e, 'Could not register you 😭');
  } finally {
    in_operation.value = false;
  }
}
onMounted(() => {
  if (need_registration.value && pseudo.value) {
    setTimeout(() => pseudo.value.focus(), 100);
  }
});
</script>

<template>
  <c-modal v-model="need_registration">
    <div class="card bg-base-100 w-fit mx-auto">
      <div class="card-title text-4xl font-bold justify-center">Registration</div>
      <div class="card-body w-fit">
        <p class="text-base">You must choose a name before playing.</p>
        <simple-input
          v-model="player_name"
          v-model:error="player_name_error"
          tabindex="1"
          ref="pseudo"
          name="pseudo"
          placeholder="Your player name..."
          autocomplete="off"
          :rules="[
            (pseudo) => (pseudo && pseudo.length >= 3) || 'Must be at least 3 chars',
            (pseudo) => pseudo.length < 30 || 'Must be max 30 chars',
          ]"
          @keydown.enter="
            if (!in_operation && !player_name_error) {
              handle_register();
            }
          "
        >
          Player Name
        </simple-input>
        <button
          class="btn btn-primary mt-2"
          :disabled="in_operation || player_name_error"
          tabindex="2"
          @click="handle_register()"
        >
          <span v-show="in_operation" class="loading loading-spinner loading-md"></span>
          Register
        </button>
      </div>
    </div>
  </c-modal>
</template>

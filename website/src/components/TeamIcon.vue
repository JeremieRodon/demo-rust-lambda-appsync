<script setup>
defineProps({
  name: String,
});
</script>

<template>
  <div>
    <div v-if="name == 'RUST'" class="h-full w-fit rounded-[15%] bg-orange-200">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-full fill-current" viewBox="0 0 32 32">
        <defs>
          <radialGradient
            id="a"
            cx="-492.035"
            cy="-883.37"
            r="13.998"
            gradientTransform="matrix(0.866, -0.5, -0.3, -0.52, 177.106, -689.033)"
            gradientUnits="userSpaceOnUse"
          >
            <stop offset="0" stop-color="#7d7d7d" />
            <stop offset="0.267" stop-color="#7e7c7a" />
            <stop offset="0.45" stop-color="#817871" />
            <stop offset="0.608" stop-color="#867162" />
            <stop offset="0.753" stop-color="#8d684c" />
            <stop offset="0.886" stop-color="#965c30" />
            <stop offset="1" stop-color="#a04f12" />
          </radialGradient>
        </defs>
        <title>file_type_rust</title>
        <path
          d="M15.124,5.3a.832.832,0,1,1,.832.832h0a.831.831,0,0,1-.832-.832M5.2,12.834a.832.832,0,1,1,.832.832h0a.832.832,0,0,1-.832-.832m19.856.039a.832.832,0,1,1,.832.832.831.831,0,0,1-.832-.832h0M7.605,14.013a.76.76,0,0,0,.386-1l-.369-.835H9.074v6.545H6.144a10.247,10.247,0,0,1-.332-3.911Zm6.074.161V12.245h3.458c.179,0,1.261.206,1.261,1.016,0,.672-.83.913-1.513.913ZM8.958,24.561a.832.832,0,1,1,.832.832.831.831,0,0,1-.832-.832h0m12.331.039a.832.832,0,1,1,.832.832.832.832,0,0,1-.832-.832h0m.257-1.887a.758.758,0,0,0-.9.584l-.418,1.949a10.249,10.249,0,0,1-8.545-.041l-.417-1.949a.759.759,0,0,0-.9-.583h0l-1.721.37a10.233,10.233,0,0,1-.89-1.049h8.374c.095,0,.158-.017.158-.1V18.928c0-.086-.063-.1-.158-.1h-2.45V16.947h2.649a1.665,1.665,0,0,1,1.629,1.412c.105.413.336,1.757.494,2.187.157.483.8,1.447,1.482,1.447h4.323a10.243,10.243,0,0,1-.949,1.1Zm4.65-7.821a10.261,10.261,0,0,1,.022,1.779H25.167c-.105,0-.148.069-.148.172v.483c0,1.136-.641,1.384-1.2,1.447-.535.06-1.128-.224-1.2-.551a3.616,3.616,0,0,0-1.671-2.808c1.03-.654,2.1-1.619,2.1-2.911A3.292,3.292,0,0,0,21.44,9.8a4.559,4.559,0,0,0-2.2-.724H8.367A10.246,10.246,0,0,1,14.1,5.84l1.282,1.344a.758.758,0,0,0,1.072.026h0l1.434-1.372a10.248,10.248,0,0,1,7.015,5l-.982,2.217a.761.761,0,0,0,.386,1Zm2.448.036-.033-.343,1.011-.943a.42.42,0,0,0-.013-.595.428.428,0,0,0-.121-.081L28.2,12.483l-.1-.334.806-1.12a.422.422,0,0,0-.13-.581.43.43,0,0,0-.133-.055l-1.363-.222-.164-.306.573-1.257a.419.419,0,0,0-.236-.544.426.426,0,0,0-.146-.029l-1.383.048L25.7,7.819l.318-1.347a.421.421,0,0,0-.343-.487.435.435,0,0,0-.144,0L24.183,6.3l-.266-.219L23.966,4.7a.421.421,0,0,0-.431-.411.426.426,0,0,0-.141.028l-1.257.573-.306-.164-.222-1.363a.421.421,0,0,0-.5-.318.43.43,0,0,0-.133.055l-1.121.806-.333-.1-.483-1.293a.421.421,0,0,0-.555-.215.442.442,0,0,0-.12.08L17.418,3.39l-.343-.033L16.347,2.18a.421.421,0,0,0-.688,0l-.728,1.177-.343.033-.943-1.012a.421.421,0,0,0-.595.015.442.442,0,0,0-.08.12L12.483,3.8l-.333.1-1.12-.8a.422.422,0,0,0-.581.13.43.43,0,0,0-.055.133l-.222,1.363-.306.164L8.608,4.317a.421.421,0,0,0-.544.239.444.444,0,0,0-.028.144l.048,1.383L7.818,6.3,6.471,5.984a.421.421,0,0,0-.487.343.435.435,0,0,0,0,.144L6.3,7.819l-.218.265L4.7,8.036a.422.422,0,0,0-.383.573L4.89,9.866l-.164.306-1.363.222a.42.42,0,0,0-.318.5.43.43,0,0,0,.055.133l.806,1.12-.1.334-1.293.483a.421.421,0,0,0-.215.555.414.414,0,0,0,.081.121l1.011.943-.033.343-1.177.728a.421.421,0,0,0,0,.688l1.177.728.033.343-1.011.943a.421.421,0,0,0,.015.595.436.436,0,0,0,.119.08l1.293.483.1.334L3.1,20.972a.421.421,0,0,0,.131.581.43.43,0,0,0,.133.055l1.363.222.164.307-.573,1.257a.422.422,0,0,0,.24.545.438.438,0,0,0,.143.028l1.383-.048.219.266-.317,1.348a.42.42,0,0,0,.341.486.4.4,0,0,0,.146,0L7.818,25.7l.266.218L8.035,27.3a.419.419,0,0,0,.429.41.413.413,0,0,0,.143-.028l1.257-.573.306.164.222,1.362a.421.421,0,0,0,.5.319.407.407,0,0,0,.133-.055l1.12-.807.334.1.483,1.292a.422.422,0,0,0,.556.214.436.436,0,0,0,.119-.08l.943-1.011.343.034.728,1.177a.422.422,0,0,0,.588.1.413.413,0,0,0,.1-.1l.728-1.177.343-.034.943,1.011a.421.421,0,0,0,.595-.015.436.436,0,0,0,.08-.119l.483-1.292.334-.1,1.12.807a.421.421,0,0,0,.581-.131.43.43,0,0,0,.055-.133l.222-1.362.306-.164,1.257.573a.421.421,0,0,0,.544-.239.438.438,0,0,0,.028-.143l-.048-1.384.265-.218,1.347.317a.421.421,0,0,0,.487-.34.447.447,0,0,0,0-.146L25.7,24.183l.218-.266,1.383.048a.421.421,0,0,0,.41-.431.4.4,0,0,0-.028-.142l-.573-1.257.164-.307,1.363-.222a.421.421,0,0,0,.319-.5.434.434,0,0,0-.056-.135l-.806-1.12.1-.334,1.293-.483a.42.42,0,0,0,.215-.554.414.414,0,0,0-.081-.121l-1.011-.943.033-.343,1.177-.728a.421.421,0,0,0,0-.688Z"
          style="fill: url(#a)"
        />
      </svg>
    </div>
    <svg
      v-if="name == 'PYTHON'"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 110 110"
      class="h-full fill-current"
    >
      <defs>
        <linearGradient id="linearGradient4671">
          <stop style="stop-color: #ffd43b; stop-opacity: 1" offset="0" id="stop4673" />
          <stop style="stop-color: #ffe873; stop-opacity: 1" offset="1" id="stop4675" />
        </linearGradient>
        <linearGradient id="linearGradient4689">
          <stop style="stop-color: #5a9fd4; stop-opacity: 1" offset="0" id="stop4691" />
          <stop style="stop-color: #306998; stop-opacity: 1" offset="1" id="stop4693" />
        </linearGradient>
        <linearGradient
          inkscape:collect="always"
          xlink:href="#linearGradient4671"
          id="linearGradient1475"
          gradientUnits="userSpaceOnUse"
          gradientTransform="matrix(0.562541,0,0,0.567972,-14.99112,-11.702371)"
          x1="150.96111"
          y1="192.35176"
          x2="112.03144"
          y2="137.27299"
        />
        <linearGradient
          inkscape:collect="always"
          xlink:href="#linearGradient4689"
          id="linearGradient1478"
          gradientUnits="userSpaceOnUse"
          gradientTransform="matrix(0.562541,0,0,0.567972,-14.99112,-11.702371)"
          x1="26.648937"
          y1="20.603781"
          x2="135.66525"
          y2="114.39767"
        />
      </defs>
      <path
        style="fill: url(#linearGradient1478); fill-opacity: 1"
        d="M 54.918785,9.1927421e-4 C 50.335132,0.02221727 45.957846,0.41313697 42.106285,1.0946693 30.760069,3.0991731 28.700036,7.2947714 28.700035,15.032169 v 10.21875 h 26.8125 v 3.40625 h -26.8125 -10.0625 c -7.792459,0 -14.6157588,4.683717 -16.7499998,13.59375 -2.46181998,10.212966 -2.57101508,16.586023 0,27.25 1.9059283,7.937852 6.4575432,13.593748 14.2499998,13.59375 h 9.21875 v -12.25 c 0,-8.849902 7.657144,-16.656248 16.75,-16.65625 h 26.78125 c 7.454951,0 13.406253,-6.138164 13.40625,-13.625 v -25.53125 c 0,-7.2663386 -6.12998,-12.7247771 -13.40625,-13.9374997 C 64.281548,0.32794397 59.502438,-0.02037903 54.918785,9.1927421e-4 Z m -14.5,8.21875012579 c 2.769547,0 5.03125,2.2986456 5.03125,5.1249996 -2e-6,2.816336 -2.261703,5.09375 -5.03125,5.09375 -2.779476,-1e-6 -5.03125,-2.277415 -5.03125,-5.09375 -10e-7,-2.826353 2.251774,-5.1249996 5.03125,-5.1249996 z"
      />
      <path
        style="fill: url(#linearGradient1475); fill-opacity: 1"
        d="m 85.637535,28.657169 v 11.90625 c 0,9.230755 -7.825895,16.999999 -16.75,17 h -26.78125 c -7.335833,0 -13.406249,6.278483 -13.40625,13.625 v 25.531247 c 0,7.266344 6.318588,11.540324 13.40625,13.625004 8.487331,2.49561 16.626237,2.94663 26.78125,0 6.750155,-1.95439 13.406253,-5.88761 13.40625,-13.625004 V 86.500919 h -26.78125 v -3.40625 h 26.78125 13.406254 c 7.792461,0 10.696251,-5.435408 13.406241,-13.59375 2.79933,-8.398886 2.68022,-16.475776 0,-27.25 -1.92578,-7.757441 -5.60387,-13.59375 -13.406241,-13.59375 z m -15.0625,64.65625 c 2.779478,3e-6 5.03125,2.277417 5.03125,5.093747 -2e-6,2.826354 -2.251775,5.125004 -5.03125,5.125004 -2.76955,0 -5.03125,-2.29865 -5.03125,-5.125004 2e-6,-2.81633 2.261697,-5.093747 5.03125,-5.093747 z"
      />
    </svg>
    <svg
      v-if="name == 'JS'"
      xmlns="http://www.w3.org/2000/svg"
      class="h-full fill-current"
      viewBox="0 0 512 512"
    >
      <rect width="512" height="512" rx="15%" fill="#f7df1e" />
      <path
        d="M324 370c10 17 24 29 47 29c20 0 33-10 33 -24c0-16 -13 -22 -35 -32l-12-5c-35-15 -58 -33 -58 -72c0-36 27 -64 70 -64c31 0 53 11 68 39l-37 24c-8-15 -17 -21 -31 -21c-14 0-23 9 -23 21c0 14 9 20 30 29l12 5c41 18 64 35 64 76c0 43-34 67 -80 67c-45 0-74 -21 -88 -49zm-170 4c8 13 14 25 31 25c16 0 26-6 26 -30V203h48v164c0 50-29 72 -72 72c-39 0-61 -20 -72 -44z"
      />
    </svg>
    <div v-if="name == 'VTL'" class="h-full w-fit rounded-[15%] bg-gray-300">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-full fill-[#262692]" viewBox="0 0 32 32">
        <path
          d="M16.245,24.187A8.352,8.352,0,1,1,24.6,15.835,8.362,8.362,0,0,1,16.245,24.187Zm0-14.762a6.41,6.41,0,1,0,6.41,6.41A6.417,6.417,0,0,0,16.245,9.425Z"
        />
        <polygon
          points="29.132 8.551 25.451 6.531 25.451 7.474 15.821 7.474 15.821 9.424 25.451 9.424 25.451 10.571 29.132 8.551"
        />
        <polygon
          points="8.984 2 6.964 5.687 7.906 5.687 7.906 15.328 9.857 15.328 9.857 5.687 11.004 5.687 8.984 2"
        />
        <polygon
          points="23.705 30 21.685 26.243 22.628 26.243 22.628 16.417 24.578 16.417 24.578 26.243 25.725 26.243 23.705 30"
        />
        <polygon
          points="2.868 23.32 6.634 21.3 6.634 22.242 16.484 22.242 16.484 24.192 6.634 24.192 6.634 25.34 2.868 23.32"
        />
      </svg>
    </div>
    <svg
      v-if="name == null"
      xmlns="http://www.w3.org/2000/svg"
      class="h-full fill-current"
      viewBox="0 0 24 24"
    >
      <path
        d="M12,21.35L10.55,20.03C5.4,15.36 2,12.27 2,8.5C2,5.41 4.42,3 7.5,3C8.17,3 8.82,3.12 9.44,3.33L13,9.35L9,14.35L12,21.35V21.35M16.5,3C19.58,3 22,5.41 22,8.5C22,12.27 18.6,15.36 13.45,20.03L12,21.35L11,14.35L15.5,9.35L12.85,4.27C13.87,3.47 15.17,3 16.5,3Z"
      />
    </svg>
  </div>
</template>

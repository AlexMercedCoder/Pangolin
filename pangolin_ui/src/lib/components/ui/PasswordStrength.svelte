<script lang="ts">
  export let password: string = '';

  interface Requirement {
    label: string;
    test: (pwd: string) => boolean;
  }

  const requirements: Requirement[] = [
    { label: 'At least 8 characters', test: (pwd) => pwd.length >= 8 },
    { label: 'Contains uppercase letter', test: (pwd) => /[A-Z]/.test(pwd) },
    { label: 'Contains lowercase letter', test: (pwd) => /[a-z]/.test(pwd) },
    { label: 'Contains number', test: (pwd) => /[0-9]/.test(pwd) },
    { label: 'Contains special character', test: (pwd) => /[^A-Za-z0-9]/.test(pwd) }
  ];

  $: metRequirements = requirements.filter(req => req.test(password));
  $: strength = password.length === 0 ? 0 : (metRequirements.length / requirements.length) * 100;
  $: strengthLabel = 
    strength === 0 ? '' :
    strength < 40 ? 'Weak' :
    strength < 80 ? 'Medium' :
    'Strong';
  $: strengthColor =
    strength === 0 ? 'bg-gray-200' :
    strength < 40 ? 'bg-red-500' :
    strength < 80 ? 'bg-yellow-500' :
    'bg-green-500';
</script>

{#if password.length > 0}
  <div class="mb-4">
    <div class="flex justify-between items-center mb-1">
      <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
        Password Strength
      </span>
      <span class="text-sm font-medium
        {strength < 40 ? 'text-red-500' : strength < 80 ? 'text-yellow-500' : 'text-green-500'}">
        {strengthLabel}
      </span>
    </div>
    
    <div class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
      <div 
        class="{strengthColor} h-full transition-all duration-300 ease-out"
        style="width: {strength}%"
      ></div>
    </div>
    
    <div class="mt-2 space-y-1">
      {#each requirements as requirement}
        <div class="flex items-center text-sm">
          {#if requirement.test(password)}
            <svg class="w-4 h-4 text-green-500 mr-2" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
            </svg>
            <span class="text-green-600 dark:text-green-400">{requirement.label}</span>
          {:else}
            <svg class="w-4 h-4 text-gray-400 mr-2" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
            </svg>
            <span class="text-gray-500 dark:text-gray-400">{requirement.label}</span>
          {/if}
        </div>
      {/each}
    </div>
  </div>
{/if}

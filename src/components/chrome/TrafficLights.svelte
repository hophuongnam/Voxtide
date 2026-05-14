<div class="flex items-center gap-2">
  {#each ['#ff5f57', '#febc2e', '#28c840'] as color}
    <div class="w-3 h-3 rounded-full" style:background={color}
         style:box-shadow="inset 0 0 0 0.5px rgba(0,0,0,0.18)"></div>
  {/each}
</div>
